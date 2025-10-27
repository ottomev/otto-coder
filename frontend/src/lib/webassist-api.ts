// WebAssist-specific API functions
import {
  WebAssistProjectSummary,
  WebAssistProjectStatus,
  WebAssistApproval,
  ApprovalDecision,
  Deliverable,
  ApiResponse,
} from 'shared/types';

const makeRequest = async (url: string, options: RequestInit = {}) => {
  const headers = {
    'Content-Type': 'application/json',
    ...(options.headers || {}),
  };

  return fetch(url, {
    ...options,
    headers,
  });
};

const handleApiResponse = async <T,>(response: Response): Promise<T> => {
  if (!response.ok) {
    let errorMessage = `Request failed with status ${response.status}`;

    try {
      const errorData = await response.json();
      if (errorData.message) {
        errorMessage = errorData.message;
      }
    } catch {
      errorMessage = response.statusText || errorMessage;
    }

    console.error('[API Error]', {
      message: errorMessage,
      status: response.status,
      response,
      endpoint: response.url,
      timestamp: new Date().toISOString(),
    });
    throw new Error(errorMessage);
  }

  const result: ApiResponse<T> = await response.json();

  if (!result.success) {
    const errorMessage = result.message || 'API request failed';
    console.error('[API Error]', {
      message: errorMessage,
      status: response.status,
      response,
      endpoint: response.url,
      timestamp: new Date().toISOString(),
    });
    throw new Error(errorMessage);
  }

  return result.data as T;
};

// WebAssist Project Management APIs
export const webAssistApi = {
  /**
   * Get all WebAssist projects with summary data
   */
  getAll: async (): Promise<WebAssistProjectSummary[]> => {
    const response = await makeRequest('/api/web-assist/projects');
    return handleApiResponse<WebAssistProjectSummary[]>(response);
  },

  /**
   * Get detailed status for a specific WebAssist project
   */
  getById: async (projectId: string): Promise<WebAssistProjectStatus> => {
    const response = await makeRequest(`/api/web-assist/projects/${projectId}`);
    return handleApiResponse<WebAssistProjectStatus>(response);
  },

  /**
   * Get all approvals for a project
   */
  getApprovals: async (projectId: string): Promise<WebAssistApproval[]> => {
    const response = await makeRequest(
      `/api/web-assist/projects/${projectId}/approvals`
    );
    return handleApiResponse<WebAssistApproval[]>(response);
  },

  /**
   * Submit an approval decision (approve/reject with feedback)
   */
  submitApproval: async (
    approvalId: string,
    decision: ApprovalDecision
  ): Promise<void> => {
    const response = await makeRequest(
      `/api/web-assist/approvals/${approvalId}`,
      {
        method: 'POST',
        body: JSON.stringify(decision),
      }
    );
    return handleApiResponse<void>(response);
  },

  /**
   * Get deliverables for a specific stage
   */
  getDeliverables: async (
    projectId: string,
    stage: string
  ): Promise<Deliverable[]> => {
    const response = await makeRequest(
      `/api/web-assist/projects/${projectId}/stages/${stage}/deliverables`
    );
    return handleApiResponse<Deliverable[]>(response);
  },

  /**
   * Trigger manual sync (admin function)
   */
  manualSync: async (projectId: string): Promise<string> => {
    const response = await makeRequest(
      `/api/web-assist/projects/${projectId}/sync`,
      {
        method: 'POST',
      }
    );
    return handleApiResponse<string>(response);
  },
};
