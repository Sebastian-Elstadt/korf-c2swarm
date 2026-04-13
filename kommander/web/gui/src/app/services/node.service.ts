import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable, map } from 'rxjs';
import type {
  AddNodeCommandRequest,
  NodeCommandEntry
} from '../models/node-command.model';
import type { NodeLogEntry } from '../models/node-log.model';
import type { Node, NodeListItem } from '../models/node.model';
import { API_BASE } from './api.constants';

/** Default until the API exposes delivery mode per node. */
const DEFAULT_DELIVERY: Node['deliveryMode'] = 'push';

function toNode(item: NodeListItem): Node {
  return { ...item, deliveryMode: DEFAULT_DELIVERY };
}

@Injectable({ providedIn: 'root' })
export class NodeService {
  private readonly http = inject(HttpClient);

  listNodes(): Observable<Node[]> {
    return this.http
      .get<NodeListItem[]>(`${API_BASE}/nodes`)
      .pipe(map((items) => items.map(toNode)));
  }

  getNodeLogs(nodeId: string): Observable<NodeLogEntry[]> {
    return this.http.get<NodeLogEntry[]>(
      `${API_BASE}/nodes/${encodeURIComponent(nodeId)}/logs`
    );
  }

  getNodeCommands(nodeId: string): Observable<NodeCommandEntry[]> {
    return this.http.get<NodeCommandEntry[]>(
      `${API_BASE}/nodes/${encodeURIComponent(nodeId)}/commands`
    );
  }

  addNodeCommand(
    nodeId: string,
    body: AddNodeCommandRequest
  ): Observable<NodeCommandEntry> {
    return this.http.post<NodeCommandEntry>(
      `${API_BASE}/nodes/${encodeURIComponent(nodeId)}/commands`,
      body
    );
  }
}
