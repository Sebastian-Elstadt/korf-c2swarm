import {
  Component,
  computed,
  effect,
  inject,
  signal
} from '@angular/core';
import { takeUntilDestroyed, toSignal } from '@angular/core/rxjs-interop';
import { catchError, interval, map, merge, of } from 'rxjs';
import {
  nodeCommandStatusLabel,
  nodeCommandTypeLabel
} from '../../models/node-command-labels';
import type {
  AddNodeCommandRequest,
  NodeCommandEntry
} from '../../models/node-command.model';
import {
  nodeLogEventTypeLabel,
  nodeLogNetworkProtocolLabel
} from '../../models/node-log-labels';
import type { NodeLogEntry } from '../../models/node-log.model';
import type { Node } from '../../models/node.model';
import { NodeService } from '../../services/node.service';

/** Heartbeat: online when last_seen_at is at most this old (offline if strictly older). */
const HEARTBEAT_MAX_AGE_MS = 10 * 60_000;

type NodesState =
  | { kind: 'loading' }
  | { kind: 'error' }
  | { kind: 'ready'; nodes: Node[] };

type LogsLoadState = 'idle' | 'loading' | 'error' | 'ready';
type CommandsLoadState = 'loading' | 'error' | 'ready';

@Component({
  selector: 'app-dashboard',
  imports: [],
  templateUrl: './dashboard.html',
  styleUrl: './dashboard.scss'
})
export class Dashboard {
  private readonly nodeService = inject(NodeService);

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
  /** 0 = Shutdown, 1 = ShellScript — matches `NodeCommandType` in domain. */
  readonly selectedCommandType = signal<0 | 1>(0);

  readonly logsState = signal<LogsLoadState>('idle');
  readonly logsEntries = signal<NodeLogEntry[]>([]);

  readonly commandsState = signal<CommandsLoadState>('loading');
  readonly commandsEntries = signal<NodeCommandEntry[]>([]);
  readonly commandSubmitBusy = signal(false);
  readonly commandActionError = signal<string | null>(null);

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
      const id = this.selectedNodeId();
      this.logsEntries.set([]);
      this.logsState.set('idle');
      this.commandsEntries.set([]);
      this.commandActionError.set(null);
      if (!id) {
        this.commandsState.set('ready');
        return;
      }
      this.loadCommands();
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

  loadCommands(): void {
    const id = this.selectedNodeId();
    if (!id) return;
    this.commandsState.set('loading');
    this.nodeService.getNodeCommands(id).subscribe({
      next: (entries) => {
        if (this.selectedNodeId() !== id) return;
        this.commandsEntries.set(entries);
        this.commandsState.set('ready');
      },
      error: () => {
        if (this.selectedNodeId() !== id) return;
        this.commandsState.set('error');
      }
    });
  }

  activateLoadLogs(): void {
    this.loadLogs();
  }

  activateLoadCommands(): void {
    this.loadCommands();
  }

  selectCommandType(t: 0 | 1): void {
    this.selectedCommandType.set(t);
    this.commandActionError.set(null);
  }

  activateSelectCommandType(t: 0 | 1): void {
    this.selectCommandType(t);
  }

  onScriptKeydown(ev: KeyboardEvent, el: HTMLElement): void {
    if (ev.key === 'Enter' && !ev.shiftKey) {
      ev.preventDefault();
      this.submitCommand(el);
    }
  }

  submitCommand(scriptEl: HTMLElement | null): void {
    const node = this.selectedNode();
    if (!node || this.commandSubmitBusy()) return;
    const type = this.selectedCommandType();
    const text = scriptEl?.textContent?.trim() ?? '';
    if (type === 1 && !text) {
      this.commandActionError.set('Enter a script body for shell script commands.');
      return;
    }
    this.commandActionError.set(null);
    const body: AddNodeCommandRequest = { command_type: type };
    if (type === 1) {
      body.text_content = text;
    }
    this.commandSubmitBusy.set(true);
    this.nodeService.addNodeCommand(node.id, body).subscribe({
      next: () => {
        this.commandSubmitBusy.set(false);
        if (scriptEl) {
          scriptEl.textContent = '';
        }
        this.loadCommands();
      },
      error: () => {
        this.commandSubmitBusy.set(false);
        this.commandActionError.set('Could not queue command.');
      }
    });
  }

  submitCommandFor(scriptLine: HTMLElement | undefined): void {
    if (this.selectedCommandType() === 0) {
      this.submitCommand(null);
    } else {
      this.submitCommand(scriptLine ?? null);
    }
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
      parts.push(nodeLogNetworkProtocolLabel(entry.network_protocol));
    }
    return parts.length > 0 ? parts.join(' · ') : null;
  }

  eventTypeLabel(eventType: number): string {
    return nodeLogEventTypeLabel(eventType);
  }

  commandStatusLabel(status: number): string {
    return nodeCommandStatusLabel(status);
  }

  commandTypeLabel(commandType: number): string {
    return nodeCommandTypeLabel(commandType);
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

  heartbeatLabel(iso: string): string {
    const r = this.relativeFromIso(iso);
    return r === 'unknown' ? 'HB: unknown' : `HB: ${r}`;
  }

  firstSeenLabel(iso: string): string {
    return this.relativeFromIso(iso);
  }

  isoShort(iso: string): string {
    return iso.length >= 19 ? iso.slice(0, 19).replace('T', ' ') : iso;
  }
}
