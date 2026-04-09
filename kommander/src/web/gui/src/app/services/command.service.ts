import { Injectable, computed, signal, type Signal } from '@angular/core';
import type { NodeDeliveryMode } from '../models/node.model';
import type { CommandStatus, QueuedCommand } from '../models/command.model';

function nextStatusAfterSend(
  current: CommandStatus,
  deliveryMode: NodeDeliveryMode
): CommandStatus {
  if (current === 'failed') return 'failed';
  if (deliveryMode === 'pull' && current === 'queued_for_poll') return 'sent';
  if (deliveryMode === 'push' && current === 'pending') return 'sent';
  return current;
}

@Injectable({ providedIn: 'root' })
export class CommandService {
  private readonly data = signal<Map<string, QueuedCommand[]>>(new Map());

  commandsView(nodeId: Signal<string | null>): Signal<QueuedCommand[]> {
    return computed(() => {
      const id = nodeId();
      if (!id) return [];
      const list = this.data().get(id) ?? [];
      return [...list].sort((a, b) => b.createdAt.localeCompare(a.createdAt));
    });
  }

  enqueue(
    nodeId: string,
    payload: string,
    kind: string,
    deliveryMode: NodeDeliveryMode
  ): void {
    const trimmed = payload.trim();
    if (!trimmed) return;

    const status: CommandStatus =
      deliveryMode === 'pull' ? 'queued_for_poll' : 'pending';

    const cmd: QueuedCommand = {
      id: crypto.randomUUID(),
      nodeId,
      payload: trimmed,
      kind: kind.trim() || 'raw',
      createdAt: new Date().toISOString(),
      status
    };

    this.data.update((m) => {
      const next = new Map(m);
      next.set(nodeId, [...(next.get(nodeId) ?? []), cmd]);
      return next;
    });
  }

  /** Demo: advance the oldest actionable command for this node (push or simulated poll). */
  simulateSend(nodeId: string, deliveryMode: NodeDeliveryMode): void {
    this.data.update((m) => {
      const next = new Map(m);
      const list = [...(next.get(nodeId) ?? [])];
      const idx = list.findIndex(
        (c) => c.status === 'pending' || c.status === 'queued_for_poll'
      );
      if (idx === -1) return m;

      const updated = { ...list[idx] };
      updated.status = nextStatusAfterSend(updated.status, deliveryMode);
      list[idx] = updated;
      next.set(nodeId, list);
      return next;
    });
  }
}
