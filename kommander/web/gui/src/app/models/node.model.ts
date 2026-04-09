/** Mirrors `NodeJson` from kommander `api.rs` plus UI-only fields until the API exposes them. */
export type NodeDeliveryMode = 'push' | 'pull';

export interface Node {
  id: string;
  nodus_id_hex: string;
  mac_addr: string;
  asym_sec_algo: number;
  asym_sec_pubkey_hex: string;
  cpu_arch: string;
  hostname: string | null;
  username: string | null;
  device_name: string | null;
  account_name: string | null;
  first_seen_at: string;
  last_seen_at: string;
  /** Server can push immediately vs commands ride the next node poll. */
  deliveryMode: NodeDeliveryMode;
}
