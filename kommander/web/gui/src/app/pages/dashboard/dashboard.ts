import {
  Component,
  computed,
  effect,
  inject,
  signal
} from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { catchError, map, merge, of } from 'rxjs';
import type { Node } from '../../models/node.model';
import { CommandService } from '../../services/command.service';
import { NodeService } from '../../services/node.service';

/** Online when last_seen_at is within this many milliseconds. */
const ONLINE_MS = 5 * 60_000;

type NodesState =
  | { kind: 'loading' }
  | { kind: 'error' }
  | { kind: 'ready'; nodes: Node[] };

@Component({
  selector: 'app-dashboard',
  imports: [],
  templateUrl: './dashboard.html',
  styleUrl: './dashboard.scss'
})
export class Dashboard {
  private readonly nodeService = inject(NodeService);
  private readonly commandService = inject(CommandService);

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
  readonly commands = this.commandService.commandsView(this.selectedNodeId);
  readonly selectedNode = computed(() => {
    const id = this.selectedNodeId();
    return this.nodes().find((n) => n.id === id) ?? null;
  });

  constructor() {
    effect(() => {
      const s = this.nodesState();
      if (s.kind !== 'ready' || s.nodes.length === 0) return;
      if (this.selectedNodeId() === null) {
        this.selectedNodeId.set(s.nodes[0].id);
      }
    });
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
    const t = new Date(node.last_seen_at).getTime();
    return Number.isFinite(t) && Date.now() - t < ONLINE_MS;
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
