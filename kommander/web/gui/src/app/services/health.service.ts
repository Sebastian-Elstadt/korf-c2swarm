import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import {
  Observable,
  catchError,
  of,
  switchMap,
  timer
} from 'rxjs';
import type { HealthResponse } from '../models/health.model';
import { API_BASE } from './api.constants';

@Injectable({ providedIn: 'root' })
export class HealthService {
  private readonly http = inject(HttpClient);

  /** One GET; maps 503 JSON body to a normal emission (same shape as 200). */
  checkHealth(): Observable<HealthResponse> {
    return this.http.get<HealthResponse>(`${API_BASE}/health`).pipe(
      catchError((err: HttpErrorResponse): Observable<HealthResponse> => {
        const body = err.error;
        if (
          body &&
          typeof body === 'object' &&
          'status' in body &&
          'database' in body
        ) {
          return of(body as HealthResponse);
        }
        return of<HealthResponse>({
          status: 'degraded',
          database: 'error'
        });
      })
    );
  }

  /** Initial check plus repeats on `periodMs` (for header / ops display). */
  pollHealth(periodMs = 30_000): Observable<HealthResponse> {
    return timer(0, periodMs).pipe(
      switchMap(() => this.checkHealth())
    );
  }
}
