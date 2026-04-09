import { Injectable } from '@angular/core';
import { Observable, of } from 'rxjs';
import { delay } from 'rxjs/operators';
import type { Node } from '../models/node.model';

const MOCK_NODES: Node[] = [
  {
    id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    nodus_id_hex: '8f3a2b1c9d0e4f5a6b7c8d9e0f1a2b3c',
    mac_addr: 'aa:bb:cc:dd:ee:01',
    asym_sec_algo: 1,
    asym_sec_pubkey_hex:
      '04a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890',
    cpu_arch: 'x86_64',
    hostname: 'edge-worker-01',
    username: 'ops',
    device_name: 'thinkstation',
    account_name: 'korf-demo',
    first_seen_at: '2026-03-01T10:00:00Z',
    last_seen_at: new Date(Date.now() - 45_000).toISOString(),
    deliveryMode: 'push'
  },
  {
    id: 'b2c3d4e5-f6a7-8901-bcde-f12345678901',
    nodus_id_hex: '1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f8',
    mac_addr: 'aa:bb:cc:dd:ee:02',
    asym_sec_algo: 1,
    asym_sec_pubkey_hex:
      '04fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321',
    cpu_arch: 'aarch64',
    hostname: 'field-kit-7',
    username: 'field',
    device_name: 'rpi-gateway',
    account_name: null,
    first_seen_at: '2026-03-10T14:22:00Z',
    last_seen_at: new Date(Date.now() - 12 * 60_000).toISOString(),
    deliveryMode: 'pull'
  },
  {
    id: 'c3d4e5f6-a7b8-9012-cdef-123456789012',
    nodus_id_hex: 'deadbeefcafe00112233445566778899aabbccddeeff0011223344556677',
    mac_addr: 'aa:bb:cc:dd:ee:03',
    asym_sec_algo: 1,
    asym_sec_pubkey_hex: '04' + 'ab'.repeat(48),
    cpu_arch: 'x86_64',
    hostname: null,
    username: null,
    device_name: 'unknown-laptop',
    account_name: 'guest',
    first_seen_at: '2026-04-01T08:00:00Z',
    last_seen_at: new Date(Date.now() - 3 * 60_000).toISOString(),
    deliveryMode: 'pull'
  }
];

@Injectable({ providedIn: 'root' })
export class NodeService {
  /** Replace with `inject(HttpClient).get<Node[]>(`${API_BASE}/nodes`)` when ready. */
  listNodes(): Observable<Node[]> {
    return of(MOCK_NODES).pipe(delay(280));
  }
}
