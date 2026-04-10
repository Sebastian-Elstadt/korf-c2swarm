/** Matches `HealthResponse` in `kommander/web/src/api/app.rs`. */
export interface HealthResponse {
  status: 'ok' | 'degraded';
  database: 'ok' | 'error';
}
