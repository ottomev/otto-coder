use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum GitHubAccountError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("GitHub account not found")]
    AccountNotFound,
    #[error("GitHub account with username '{0}' already exists")]
    UsernameExists(String),
    #[error("No authentication token provided (pat or oauth_token required)")]
    NoTokenProvided,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct GitHubAccount {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)] // Don't expose tokens in API responses
    pub oauth_token: Option<String>,
    #[serde(skip_serializing)] // Don't expose tokens in API responses
    pub pat: Option<String>,
    pub primary_email: Option<String>,

    #[ts(type = "Date")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "Date")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GitHubAccountSafe {
    pub id: Uuid,
    pub username: String,
    pub primary_email: Option<String>,
    pub has_token: bool,
    #[ts(type = "Date")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "Date")]
    pub updated_at: DateTime<Utc>,
}

impl From<GitHubAccount> for GitHubAccountSafe {
    fn from(account: GitHubAccount) -> Self {
        Self {
            id: account.id,
            username: account.username,
            primary_email: account.primary_email,
            has_token: account.oauth_token.is_some() || account.pat.is_some(),
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateGitHubAccount {
    pub username: String,
    pub oauth_token: Option<String>,
    pub pat: Option<String>,
    pub primary_email: Option<String>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct UpdateGitHubAccount {
    pub username: Option<String>,
    pub oauth_token: Option<String>,
    pub pat: Option<String>,
    pub primary_email: Option<String>,
}

impl GitHubAccount {
    /// Get the authentication token (prefers PAT over OAuth token)
    pub fn token(&self) -> Option<String> {
        self.pat
            .as_deref()
            .or(self.oauth_token.as_deref())
            .map(|s| s.to_string())
    }

    /// Find all GitHub accounts
    pub async fn find_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            GitHubAccount,
            r#"
            SELECT
                id as "id!: Uuid",
                username,
                oauth_token,
                pat,
                primary_email,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            FROM github_accounts
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool)
        .await
    }

    /// Find a GitHub account by ID
    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            GitHubAccount,
            r#"
            SELECT
                id as "id!: Uuid",
                username,
                oauth_token,
                pat,
                primary_email,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            FROM github_accounts
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    /// Find a GitHub account by username
    pub async fn find_by_username(
        pool: &SqlitePool,
        username: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            GitHubAccount,
            r#"
            SELECT
                id as "id!: Uuid",
                username,
                oauth_token,
                pat,
                primary_email,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            FROM github_accounts
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(pool)
        .await
    }

    /// Create a new GitHub account
    pub async fn create(
        pool: &SqlitePool,
        data: &CreateGitHubAccount,
    ) -> Result<Self, GitHubAccountError> {
        // Validate that at least one token is provided
        if data.oauth_token.is_none() && data.pat.is_none() {
            return Err(GitHubAccountError::NoTokenProvided);
        }

        // Check if username already exists
        if let Some(_existing) = Self::find_by_username(pool, &data.username).await? {
            return Err(GitHubAccountError::UsernameExists(
                data.username.clone(),
            ));
        }

        let id = Uuid::new_v4();

        sqlx::query_as!(
            GitHubAccount,
            r#"
            INSERT INTO github_accounts (id, username, oauth_token, pat, primary_email)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                id as "id!: Uuid",
                username,
                oauth_token,
                pat,
                primary_email,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            "#,
            id,
            data.username,
            data.oauth_token,
            data.pat,
            data.primary_email
        )
        .fetch_one(pool)
        .await
        .map_err(GitHubAccountError::from)
    }

    /// Update a GitHub account
    pub async fn update(
        pool: &SqlitePool,
        id: Uuid,
        data: &UpdateGitHubAccount,
    ) -> Result<Self, GitHubAccountError> {
        // Fetch existing account
        let existing = Self::find_by_id(pool, id)
            .await?
            .ok_or(GitHubAccountError::AccountNotFound)?;

        // Check if new username conflicts with another account
        if let Some(ref new_username) = data.username {
            if new_username != &existing.username {
                if let Some(_conflicting) = Self::find_by_username(pool, new_username).await? {
                    return Err(GitHubAccountError::UsernameExists(new_username.clone()));
                }
            }
        }

        let username = data.username.as_ref().unwrap_or(&existing.username);
        let oauth_token = data
            .oauth_token
            .as_ref()
            .or(existing.oauth_token.as_ref());
        let pat = data.pat.as_ref().or(existing.pat.as_ref());
        let primary_email = data
            .primary_email
            .as_ref()
            .or(existing.primary_email.as_ref());

        sqlx::query_as!(
            GitHubAccount,
            r#"
            UPDATE github_accounts
            SET
                username = $2,
                oauth_token = $3,
                pat = $4,
                primary_email = $5,
                updated_at = datetime('now', 'subsec')
            WHERE id = $1
            RETURNING
                id as "id!: Uuid",
                username,
                oauth_token,
                pat,
                primary_email,
                created_at as "created_at!: DateTime<Utc>",
                updated_at as "updated_at!: DateTime<Utc>"
            "#,
            id,
            username,
            oauth_token,
            pat,
            primary_email
        )
        .fetch_one(pool)
        .await
        .map_err(GitHubAccountError::from)
    }

    /// Delete a GitHub account
    pub async fn delete(pool: &SqlitePool, id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!("DELETE FROM github_accounts WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Check if a GitHub account exists
    pub async fn exists(pool: &SqlitePool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!: i64"
            FROM github_accounts
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(result.count > 0)
    }

    /// Count total GitHub accounts
    pub async fn count(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!: i64" FROM github_accounts"#
        )
        .fetch_one(pool)
        .await
    }
}
