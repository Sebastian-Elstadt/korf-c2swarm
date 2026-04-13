/**
 * Labels for `NodeCommandStatus` / `NodeCommandType` in
 * `kommander/domain/src/node/command.rs` (keep in sync with DB/API).
 *
 * Status: Queued=0, Executing=1, Completed=2; Cancelled is 3 in DB mapping.
 */

export function nodeCommandStatusLabel(status: number): string {
  switch (status) {
    case 0:
      return 'Queued';
    case 1:
      return 'Executing';
    case 2:
      return 'Completed';
    case 3:
    case 4:
      return 'Cancelled';
    default:
      return `Unknown (${status})`;
  }
}

export function nodeCommandTypeLabel(commandType: number): string {
  switch (commandType) {
    case 0:
      return 'Shutdown';
    case 1:
      return 'Shell script';
    default:
      return `Unknown (${commandType})`;
  }
}
