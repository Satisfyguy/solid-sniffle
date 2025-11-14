#!/usr/bin/env python3
"""
Multisig Instrumentation Log Analyzer

Analyzes JSON instrumentation logs from the Monero Marketplace escrow system
to identify race conditions, RPC cache pollution, and state corruption.

Usage:
    python tools/analyze_escrow_json.py escrow_abc123.json
    python tools/analyze_escrow_json.py --compare escrow_success.json escrow_failed.json
    python tools/analyze_escrow_json.py --timeline escrow_abc123.json
    python tools/analyze_escrow_json.py --diff-snapshots escrow_abc123.json
"""

import json
import sys
from collections import defaultdict
from datetime import datetime
from typing import Dict, List, Any
import argparse


def load_events(json_file: str) -> List[Dict[str, Any]]:
    """Load events from JSON file"""
    with open(json_file) as f:
        return json.load(f)


def format_timestamp(ts_ms: int) -> str:
    """Convert millisecond timestamp to readable format"""
    return datetime.fromtimestamp(ts_ms / 1000.0).strftime('%H:%M:%S.%f')[:-3]


def analyze_timeline(events: List[Dict[str, Any]]):
    """Print chronological timeline of events"""
    print("=" * 80)
    print("EVENT TIMELINE")
    print("=" * 80)

    if not events:
        print("No events found.")
        return

    start_ts = events[0]['timestamp']

    for event in events:
        ts = event['timestamp']
        relative_ms = ts - start_ts
        event_type = event['event_type']
        role = event.get('role', '?')
        port = event.get('rpc_port', '?')

        # Format different event types
        if event_type in ['RPC_CALL_START', 'RPC_CALL_END', 'RPC_CALL_ERROR']:
            method = event['details'].get('method', 'unknown')
            if event_type == 'RPC_CALL_END':
                duration = event['details'].get('duration_ms', '?')
                success = event['details'].get('success', False)
                status = "✓" if success else "✗"
                print(f"[+{relative_ms:6d}ms] {event_type:30} role={role:7} port={port:5} method={method:20} {status} {duration}ms")
            else:
                print(f"[+{relative_ms:6d}ms] {event_type:30} role={role:7} port={port:5} method={method:20}")

        elif 'SNAPSHOT' in event_type:
            snapshot = event['details']
            is_multisig = snapshot.get('is_multisig', False)
            balance = snapshot.get('balance', [0, 0])
            address_hash = snapshot.get('address_hash', 'unknown')[:16]
            print(f"[+{relative_ms:6d}ms] {event_type:30} role={role:7} multisig={is_multisig} balance={balance[0]:12} addr={address_hash}")

        elif event_type == 'ERROR_FINAL':
            error_msg = event['details'].get('error', 'unknown')
            print(f"[+{relative_ms:6d}ms] {event_type:30} role={role:7} ERROR: {error_msg}")

        else:
            print(f"[+{relative_ms:6d}ms] {event_type:30} role={role:7}")

    print()


def analyze_rpc_calls(events: List[Dict[str, Any]]):
    """Analyze RPC call statistics"""
    print("=" * 80)
    print("RPC CALL STATISTICS")
    print("=" * 80)

    rpc_calls = [e for e in events if 'RPC_CALL' in e['event_type']]

    if not rpc_calls:
        print("No RPC calls recorded.")
        return

    by_method = defaultdict(list)

    for call in rpc_calls:
        if call['event_type'] == 'RPC_CALL_END':
            method = call['details'].get('method')
            duration = call['details'].get('duration_ms')
            success = call['details'].get('success', False)

            if method and duration is not None:
                by_method[method].append({
                    'duration': duration,
                    'success': success,
                    'role': call.get('role'),
                    'port': call.get('rpc_port'),
                })

    for method, calls in sorted(by_method.items()):
        durations = [c['duration'] for c in calls]
        successes = sum(1 for c in calls if c['success'])
        failures = len(calls) - successes

        avg = sum(durations) / len(durations)
        max_d = max(durations)
        min_d = min(durations)

        print(f"\n{method}:")
        print(f"  Calls:    {len(calls)} ({successes} success, {failures} failures)")
        print(f"  Duration: avg={avg:.0f}ms, min={min_d}ms, max={max_d}ms")

        # Show per-role breakdown
        by_role = defaultdict(list)
        for call in calls:
            by_role[call['role']].append(call['duration'])

        for role, role_durations in sorted(by_role.items()):
            role_avg = sum(role_durations) / len(role_durations)
            print(f"    {role:7}: {len(role_durations)} calls, avg={role_avg:.0f}ms")

    print()


def analyze_snapshots(events: List[Dict[str, Any]]):
    """Analyze wallet state snapshots for divergence"""
    print("=" * 80)
    print("WALLET STATE SNAPSHOTS")
    print("=" * 80)

    snapshots = [e for e in events if 'SNAPSHOT' in e['event_type']]

    if not snapshots:
        print("No snapshots recorded.")
        return

    # Group by role
    by_role = defaultdict(list)
    for snap in snapshots:
        role = snap.get('role')
        by_role[role].append(snap)

    # Analyze each role's progression
    for role in sorted(by_role.keys()):
        print(f"\n{role.upper()} PROGRESSION:")
        role_snaps = by_role[role]

        for i, snap in enumerate(role_snaps):
            event_type = snap['event_type']
            details = snap['details']

            is_multisig = details.get('is_multisig', False)
            balance = details.get('balance', [0, 0])
            address_hash = details.get('address_hash', 'unknown')[:16]
            collection_time = details.get('collection_time_ms', 0)

            print(f"  [{i+1}] {event_type:30}")
            print(f"      is_multisig:     {is_multisig}")
            print(f"      balance:         {balance[0]} (unlocked: {balance[1]})")
            print(f"      address_hash:    {address_hash}")
            print(f"      collection_time: {collection_time}ms")

            # Detect unexpected state changes
            if i > 0:
                prev_snap = role_snaps[i-1]['details']
                if prev_snap.get('is_multisig') != is_multisig:
                    print(f"      ⚠️  MULTISIG STATE CHANGED: {prev_snap.get('is_multisig')} → {is_multisig}")
                if prev_snap.get('address_hash') != address_hash:
                    print(f"      ⚠️  ADDRESS CHANGED")

    print()


def analyze_errors(events: List[Dict[str, Any]]):
    """Analyze error events"""
    print("=" * 80)
    print("ERRORS & ANOMALIES")
    print("=" * 80)

    errors = [e for e in events if e['event_type'] in ['ERROR_FINAL', 'RPC_CALL_ERROR', 'CACHE_POLLUTION_DETECTED']]

    if not errors:
        print("✓ No errors recorded.")
        return

    for err in errors:
        ts = err['timestamp']
        event_type = err['event_type']
        role = err.get('role', 'unknown')

        print(f"\n[{format_timestamp(ts)}] {event_type} (role={role})")

        if event_type == 'ERROR_FINAL':
            error_msg = err['details'].get('error', 'unknown')
            context = err['details'].get('context', {})
            print(f"  Error: {error_msg}")
            print(f"  Context: {json.dumps(context, indent=4)}")

        elif event_type == 'RPC_CALL_ERROR':
            method = err['details'].get('method', 'unknown')
            print(f"  RPC method failed: {method}")

        elif event_type == 'CACHE_POLLUTION_DETECTED':
            reason = err['details'].get('reason', 'unknown')
            print(f"  Reason: {reason}")

    print()


def compare_escrows(file1: str, file2: str):
    """Compare two escrow sessions side-by-side"""
    print("=" * 80)
    print(f"COMPARING: {file1} vs {file2}")
    print("=" * 80)

    events1 = load_events(file1)
    events2 = load_events(file2)

    print(f"\nEvent counts: {len(events1)} vs {len(events2)}")

    # Compare event type distribution
    types1 = defaultdict(int)
    types2 = defaultdict(int)

    for e in events1:
        types1[e['event_type']] += 1
    for e in events2:
        types2[e['event_type']] += 1

    all_types = set(types1.keys()) | set(types2.keys())

    print("\nEvent type distribution:")
    print(f"{'Event Type':<30} {'File 1':>10} {'File 2':>10} {'Diff':>10}")
    print("-" * 62)

    for event_type in sorted(all_types):
        count1 = types1.get(event_type, 0)
        count2 = types2.get(event_type, 0)
        diff = count2 - count1
        diff_str = f"+{diff}" if diff > 0 else str(diff)
        print(f"{event_type:<30} {count1:>10} {count2:>10} {diff_str:>10}")

    # Find first divergence point
    print("\n" + "=" * 80)
    print("FIRST DIVERGENCE POINT")
    print("=" * 80)

    min_len = min(len(events1), len(events2))
    divergence_found = False

    for i in range(min_len):
        e1 = events1[i]
        e2 = events2[i]

        if e1['event_type'] != e2['event_type'] or e1.get('role') != e2.get('role'):
            print(f"\nDivergence at event #{i+1}:")
            print(f"  File 1: [{e1['event_type']}] role={e1.get('role')}")
            print(f"  File 2: [{e2['event_type']}] role={e2.get('role')}")
            divergence_found = True
            break

    if not divergence_found:
        if len(events1) != len(events2):
            print(f"\nNo event divergence in first {min_len} events.")
            print(f"Length mismatch: {len(events1)} vs {len(events2)}")
        else:
            print("\n✓ Event sequences are identical.")

    print()


def main():
    parser = argparse.ArgumentParser(
        description='Analyze multisig instrumentation logs',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )

    parser.add_argument('file', help='JSON instrumentation file to analyze')
    parser.add_argument('--compare', metavar='FILE2', help='Compare with another escrow file')
    parser.add_argument('--timeline', action='store_true', help='Show detailed timeline (default)')
    parser.add_argument('--rpc-only', action='store_true', help='Show only RPC statistics')
    parser.add_argument('--snapshots-only', action='store_true', help='Show only snapshot analysis')
    parser.add_argument('--errors-only', action='store_true', help='Show only errors')

    args = parser.parse_args()

    if args.compare:
        compare_escrows(args.file, args.compare)
        return

    # Load events
    events = load_events(args.file)

    if not events:
        print(f"No events found in {args.file}")
        return

    trace_id = events[0].get('trace_id', 'unknown')
    print(f"\n{'=' * 80}")
    print(f"ESCROW ANALYSIS: {args.file}")
    print(f"Trace ID: {trace_id}")
    print(f"Total events: {len(events)}")
    print(f"{'=' * 80}\n")

    # Determine what to show
    show_all = not (args.rpc_only or args.snapshots_only or args.errors_only)

    if show_all or args.timeline:
        analyze_timeline(events)

    if show_all or args.rpc_only:
        analyze_rpc_calls(events)

    if show_all or args.snapshots_only:
        analyze_snapshots(events)

    if show_all or args.errors_only:
        analyze_errors(events)


if __name__ == '__main__':
    main()
