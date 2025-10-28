#!/usr/bin/env python3
"""
Manual WebAssist Webhook Trigger
Manually trigger the project.created webhook for testing
"""

import hashlib
import hmac
import json
import sys
import requests
from uuid import uuid4

# Configuration
OTTO_CODER_URL = "https://coder.otto.lk/api/web-assist/webhook"
WEBHOOK_SECRET = "your-webhook-secret-here"  # TODO: Replace with actual secret from config/web-assist.toml

# Sample project data - Replace with actual project data from Supabase
SAMPLE_PROJECT = {
    "event": "project.created",
    "data": {
        "project_id": "00000000-0000-0000-0000-000000000000",  # TODO: Replace with actual Supabase project ID
        "project_number": "WA-2025-001",  # TODO: Replace with actual project number
        "company_name": "Test Company",  # TODO: Replace with actual company name
        "wizard_completion_id": "00000000-0000-0000-0000-000000000000",  # TODO: Replace with actual wizard completion ID
        "is_rush_delivery": False
    }
}


def compute_hmac_signature(payload: bytes, secret: str) -> str:
    """Compute HMAC-SHA256 signature for webhook payload"""
    mac = hmac.new(secret.encode(), payload, hashlib.sha256)
    return mac.hexdigest()


def send_webhook(project_data: dict, webhook_secret: str, target_url: str):
    """Send webhook to Otto Coder"""
    # Convert to JSON
    payload = json.dumps(project_data).encode('utf-8')

    # Compute signature
    signature = compute_hmac_signature(payload, webhook_secret)

    # Prepare headers
    headers = {
        'Content-Type': 'application/json',
        'X-Supabase-Signature': signature
    }

    print(f"Sending webhook to {target_url}...")
    print(f"Project ID: {project_data['data']['project_id']}")
    print(f"Company: {project_data['data']['company_name']}")
    print(f"Signature: {signature[:20]}...")

    try:
        response = requests.post(target_url, data=payload, headers=headers, timeout=30)

        print(f"\nResponse Status: {response.status_code}")
        print(f"Response Body: {response.text}")

        if response.status_code == 200:
            print("\n✅ Webhook sent successfully!")
        else:
            print(f"\n❌ Webhook failed with status {response.status_code}")

        return response

    except Exception as e:
        print(f"\n❌ Error sending webhook: {e}")
        return None


def query_supabase_project():
    """
    Query Supabase to get actual project data
    You'll need to replace this with actual Supabase credentials
    """
    print("TODO: Query Supabase for actual project data")
    print("For now, edit the SAMPLE_PROJECT dictionary above with real data\n")
    return None


if __name__ == "__main__":
    print("=" * 60)
    print("WebAssist Manual Webhook Trigger")
    print("=" * 60)
    print()

    # Check if webhook secret is configured
    if WEBHOOK_SECRET == "your-webhook-secret-here":
        print("⚠️  WARNING: Webhook secret not configured!")
        print("Please edit this script and set WEBHOOK_SECRET to the value from config/web-assist.toml")
        print()

    # Check if project data is configured
    if SAMPLE_PROJECT["data"]["project_id"] == "00000000-0000-0000-0000-000000000000":
        print("⚠️  WARNING: Project data not configured!")
        print("Please edit SAMPLE_PROJECT with actual data from your Supabase projects table")
        print()
        print("You need to provide:")
        print("  - project_id: UUID from Supabase projects.id")
        print("  - project_number: e.g., 'WA-2025-001'")
        print("  - company_name: Client company name")
        print("  - wizard_completion_id: UUID from wizard_completions.id")
        print("  - is_rush_delivery: true/false")
        print()

        proceed = input("Do you want to proceed anyway? (yes/no): ")
        if proceed.lower() != "yes":
            print("Exiting...")
            sys.exit(0)

    # Send webhook
    send_webhook(SAMPLE_PROJECT, WEBHOOK_SECRET, OTTO_CODER_URL)

    print("\n" + "=" * 60)
    print("Check Otto Coder logs for webhook processing details")
    print("Check database: SELECT * FROM web_assist_projects;")
    print("=" * 60)
