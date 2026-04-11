/**
 * Labels for `NodeLogEventType` / `NodeLogNetworkProtocol` in
 * `kommander/domain/src/node.rs` (must stay in sync with DB values).
 */

export function nodeLogEventTypeLabel(eventType: number): string {
  switch (eventType) {
    case 0:
      return 'Unknown';
    case 1:
      return 'Heartbeat';
    case 2:
      return 'Registration';
    default:
      return `Unknown (${eventType})`;
  }
}

export function nodeLogNetworkProtocolLabel(protocol: number): string {
  switch (protocol) {
    case 0:
      return 'Unknown';
    case 1:
      return 'UDP';
    default:
      return `Unknown (${protocol})`;
  }
}
