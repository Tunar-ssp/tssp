import { expect, afterAll, afterEach, beforeAll, vi } from 'vitest';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
Object.defineProperty(global, 'localStorage', { value: localStorageMock });

// Create a mock MSW server (can be extended per test)
export const mockServer = setupServer(
  http.get('/api/v1/*', () => {
    return HttpResponse.json({});
  })
);

beforeAll(() => mockServer.listen({ onUnhandledRequest: 'warn' }));
afterEach(() => {
  mockServer.resetHandlers();
  vi.clearAllMocks();
});
afterAll(() => mockServer.close());

