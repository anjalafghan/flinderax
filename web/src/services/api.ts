const DEV_API_URL = 'http://0.0.0.0:3000';
const PROD_API_URL = 'https://flinderax-backend.fly.dev';

const BASE_URL = import.meta.env.VITE_API_URL || (import.meta.env.DEV ? DEV_API_URL : PROD_API_URL);

interface ApiResponse<T> {
    data: T;
    status: number;
    statusText: string;
}

const api = {
    get: async <T>(url: string): Promise<ApiResponse<T>> => {
        const token = localStorage.getItem('token');
        const headers: HeadersInit = {
            'Content-Type': 'application/json',
        };
        if (token) {
            headers['Authorization'] = `Bearer ${token}`;
        }

        const fullUrl = url.startsWith('http') ? url : `${BASE_URL}${url}`;

        const response = await fetch(fullUrl, {
            method: 'GET',
            headers,
        });

        if (!response.ok) {
            // Attempt to read error message from body if possible
            const errorBody = await response.text().catch(() => '');
            throw new Error(errorBody || `Request failed with status ${response.status}`);
        }

        const data = await response.json();
        return {
            data,
            status: response.status,
            statusText: response.statusText,
        };
    },

    // Implement other methods if needed, but trace only showed GET being critical for LCP/loading.
    // We'll add post just in case it's used elsewhere (AuthPage likely uses it)
    post: async <T>(url: string, body: any): Promise<ApiResponse<T>> => {
        const token = localStorage.getItem('token');
        const headers: HeadersInit = {
            'Content-Type': 'application/json',
        };
        if (token) {
            headers['Authorization'] = `Bearer ${token}`;
        }

        const fullUrl = url.startsWith('http') ? url : `${BASE_URL}${url}`;

        const response = await fetch(fullUrl, {
            method: 'POST',
            headers,
            body: JSON.stringify(body),
        });

        if (!response.ok) {
            const errorBody = await response.text().catch(() => '');
            throw new Error(errorBody || `Request failed with status ${response.status}`);
        }

        const data = await response.json();
        return {
            data,
            status: response.status,
            statusText: response.statusText,
        };
    }
};

export default api;
