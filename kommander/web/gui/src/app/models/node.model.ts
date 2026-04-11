/** Matches `NodeListItem` in `kommander/web/src/api/nodes.rs` (JSON body). */
export interface NodeListItem {
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
  /** RFC3339 from API; last reported host clock at heartbeat. */
  host_local_time: string | null;
}

export type NodeDeliveryMode = 'push' | 'pull';

/** API row plus UI-only fields until the API exposes them. */
export interface Node extends NodeListItem {
  deliveryMode: NodeDeliveryMode;
}
