import {
  Component,
  computed,
  effect,
  inject,
  signal
} from '@angular/core';
import { takeUntilDestroyed, toSignal } from '@angular/core/rxjs-interop';
import { catchError, interval, map, merge, of } from 'rxjs';
import type { NodeLogEntry } from '../../models/node-log.model';
import type { Node } from '../../models/node.model';
import { CommandService } from '../../services/command.service';
import { NodeService } from '../../services/node.service';

/** Heartbeat: online when last_seen_at is at most this old (offline if strictly older). */
const HEARTBEAT_MAX_AGE_MS = 10 * 60_000;

type NodesState =
  | { kind: 'loading' }
  | { kind: 'error' }
  | { kind: 'ready'; nodes: Node[] };

type LogsLoadState = 'idle' | 'loading' | 'error' | 'ready';

@Component({
  selector: 'app-dashboard',
  imports: [],
  templateUrl: './dashboard.html',
  styleUrl: './dashboard.scss'
})
export class Dashboard {
  private readonly nodeService = inject(NodeService);
  private readonly commandService = inject(CommandService);

  /** Advances every minute so "HB: N minutes ago" stays accurate while the view is open. */
  private readonly now = signal(Date.now());

  readonly nodesState = toSignal(
    merge(
      of<NodesState>({ kind: 'loading' }),
      this.nodeService.listNodes().pipe(
        map((nodes): NodesState => ({ kind: 'ready', nodes })),
        catchError(() => of<NodesState>({ kind: 'error' }))
      )
    ),
    { initialValue: { kind: 'loading' } as NodesState }
  );

  readonly nodes = computed(() => {
    const s = this.nodesState();
    return s.kind === 'ready' ? s.nodes : [];
  });

  readonly selectedNodeId = signal<string | null>(null);
  readonly logsState = signal<LogsLoadState>('idle');
  readonly logsEntries = signal<NodeLogEntry[]>([]);
  readonly commands = this.commandService.commandsView(this.selectedNodeId);
  readonly selectedNode = computed(() => {
    const id = this.selectedNodeId();
    return this.nodes().find((n) => n.id === id) ?? null;
  });

  constructor() {
    interval(60_000)
      .pipe(takeUntilDestroyed())
      .subscribe(() => this.now.set(Date.now()));

    effect(() => {
      const s = this.nodesState();
      if (s.kind !== 'ready' || s.nodes.length === 0) return;
      if (this.selectedNodeId() === null) {
        this.selectedNodeId.set(s.nodes[0].id);
      }
    });

    effect(() => {
      this.selectedNodeId();
      this.logsEntries.set([]);
      this.logsState.set('idle');
    });
  }

  loadLogs(): void {
    const id = this.selectedNodeId();
    if (!id) return;
    this.logsState.set('loading');
    this.nodeService.getNodeLogs(id).subscribe({
      next: (entries) => {
        if (this.selectedNodeId() !== id) return;
        this.logsEntries.set(entries);
        this.logsState.set('ready');
      },
      error: () => {
        if (this.selectedNodeId() !== id) return;
        this.logsState.set('error');
      }
    });
  }

  activateLoadLogs(): void {
    this.loadLogs();
  }

  logNetworkMeta(entry: NodeLogEntry): string | null {
    const parts: string[] = [];
    if (entry.ipv4_addr) {
      parts.push(entry.ipv4_addr);
    }
    if (entry.network_port != null) {
      parts.push(String(entry.network_port));
    }
    if (entry.network_protocol != null) {
      parts.push(`proto ${entry.network_protocol}`);
    }
    return parts.length > 0 ? parts.join(' · ') : null;
  }

  selectNode(id: string): void {
    this.selectedNodeId.set(id);
  }

  displayName(node: Node): string {
    return (
      node.hostname ??
      node.device_name ??
      `${node.nodus_id_hex.slice(0, 10)}…`
    );
  }

  isOnline(node: Node): boolean {
    void this.now();
    const t = new Date(node.last_seen_at).getTime();
    return (
      Number.isFinite(t) && Date.now() - t <= HEARTBEAT_MAX_AGE_MS
    );
  }

  /**
   * Relative time from ISO timestamp, e.g. "2 minutes ago". Uses `now` tick.
   */
  private relativeFromIso(iso: string): string {
    void this.now();
    const t = new Date(iso).getTime();
    if (!Number.isFinite(t)) {
      return 'unknown';
    }
    const diffMs = Date.now() - t;
    const past = diffMs >= 0;
    const absSec = Math.round(Math.abs(diffMs) / 1000);
    if (absSec < 60) {
      return `${absSec} second${absSec === 1 ? '' : 's'} ${past ? 'ago' : 'from now'}`;
    }
    if (absSec < 3600) {
      const m = Math.floor(absSec / 60);
      return `${m} minute${m === 1 ? '' : 's'} ${past ? 'ago' : 'from now'}`;
    }
    if (absSec < 86400) {
      const h = Math.floor(absSec / 3600);
      return `${h} hour${h === 1 ? '' : 's'} ${past ? 'ago' : 'from now'}`;
    }
    const d = Math.floor(absSec / 86400);
    return `${d} day${d === 1 ? '' : 's'} ${past ? 'ago' : 'from now'}`;
  }

  /**
   * Human-readable heartbeat from API `last_seen_at`, e.g. "HB: 2 minutes ago".
   */
  heartbeatLabel(iso: string): string {
    const r = this.relativeFromIso(iso);
    return r === 'unknown' ? 'HB: unknown' : `HB: ${r}`;
  }

  /** Relative label for `first_seen_at` in the detail panel (same rules as heartbeat text). */
  firstSeenLabel(iso: string): string {
    return this.relativeFromIso(iso);
  }

  onComposerKeydown(ev: KeyboardEvent, el: HTMLElement): void {
    if (ev.key === 'Enter') {
      ev.preventDefault();
      this.enqueueFrom(el);
    }
  }

  enqueueFrom(lineEl: HTMLElement): void {
    const node = this.selectedNode();
    if (!node) return;
    const text = lineEl.textContent?.trim() ?? '';
    if (!text) return;
    this.commandService.enqueue(node.id, text, 'exec', node.deliveryMode);
    lineEl.textContent = '';
  }

  simulateDelivery(): void {
    const node = this.selectedNode();
    if (!node) return;
    this.commandService.simulateSend(node.id, node.deliveryMode);
  }

  activateEnqueue(lineEl: HTMLElement): void {
    this.enqueueFrom(lineEl);
  }

  activateSimulate(): void {
    this.simulateDelivery();
  }

  isoShort(iso: string): string {
    return iso.length >= 19 ? iso.slice(0, 19).replace('T', ' ') : iso;
  }
}
