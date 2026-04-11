/** Matches `NodeLogEntryItem` in `kommander/web/src/api/nodes.rs`. */
export interface NodeLogEntry {
  id: string;
  created_at: string;
  event_type: number;
  text_content: string | null;
  ipv4_addr: string | null;
  network_port: number | null;
  network_protocol: number | null;
}
