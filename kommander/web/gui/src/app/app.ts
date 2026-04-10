import { Component, computed, inject } from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { RouterOutlet } from '@angular/router';
import type { HealthResponse } from './models/health.model';
import { HealthService } from './services/health.service';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet],
  templateUrl: './app.html',
  styleUrl: './app.scss'
})
export class App {
  private readonly healthService = inject(HealthService);

  readonly health = toSignal(this.healthService.pollHealth(), {
    initialValue: {
      status: 'degraded',
      database: 'error'
    } satisfies HealthResponse
  });

  readonly healthOk = computed(
    () => this.health().status === 'ok' && this.health().database === 'ok'
  );

  readonly healthLabel = computed(() =>
    this.healthOk() ? 'API and database healthy' : 'API or database degraded'
  );
}
