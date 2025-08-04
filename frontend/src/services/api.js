import axios from 'axios';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080';

// Create axios instance with default config
const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add auth token to requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('authToken');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle auth errors
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('authToken');
      localStorage.removeItem('username');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// Auth API
export const authAPI = {
  register: (userData) => api.post('/register', userData),
  login: (credentials) => api.post('/login', credentials),
};

// Password API
export const passwordAPI = {
  getAll: () => api.get('/passwords'),
  create: (passwordData) => api.post('/passwords', passwordData),
  update: (id, passwordData) => api.put(`/passwords/${id}`, passwordData),
  move: (id, moveData) => api.put(`/passwords/${id}/move`, moveData),
  delete: (id) => api.delete(`/passwords/${id}`),
  generateOTP: (id) => api.get(`/passwords/${id}/otp`),
  share: (id, shareData) => api.post(`/passwords/${id}/share`, shareData),
  getSharedPasswords: () => api.get('/shared/passwords'),
};

// Share API
export const shareAPI = {
  getSharedItems: () => api.get('/shared'),
  removeShare: (shareId) => api.delete(`/shares/${shareId}`),
};

// Folder API
export const folderAPI = {
  getAll: () => api.get('/folders'),
  create: (folderData) => api.post('/folders', folderData),
  delete: (id) => api.delete(`/folders/${id}`),
  share: (id, shareData) => api.post(`/folders/${id}/share`, shareData),
};

export default api;