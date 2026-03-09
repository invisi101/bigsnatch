#!/usr/bin/env python3
"""BigSnatch CLI — tail live connection events from the daemon.

Requires: pip install grpcio grpcio-tools
Generate stubs: python -m grpc_tools.protoc -I proto --python_out=tools --grpc_python_out=tools proto/snitchster.proto
"""

import sys
import os

# Allow importing generated stubs from the same directory
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import grpc
import snitchster_pb2 as pb
import snitchster_pb2_grpc as rpc
from datetime import datetime

SOCKET = "unix:///run/snitchster.sock"
PROTO = {0: "TCP", 1: "UDP"}

def main():
    channel = grpc.insecure_channel(SOCKET, options=[
        ("grpc.default_authority", "localhost"),
    ])
    stub = rpc.MonitorStub(channel)

    print(f"Connected to {SOCKET}", flush=True)
    print(f"{'Time':<10} {'Proto':<5} {'Process':<22} {'PID':>6}  {'Destination':<44} {'Port':>5}  Domain", flush=True)
    print("-" * 130, flush=True)

    try:
        for event in stub.Subscribe(pb.SubscribeRequest(), wait_for_ready=True):
            if event.HasField("connection"):
                c = event.connection
                ts = datetime.fromtimestamp(c.timestamp_ns / 1e9).strftime("%H:%M:%S")
                proto = PROTO.get(c.protocol, "?")
                domain = c.domain or ""
                print(f"{ts:<10} {proto:<5} {c.process_name:<22} {c.pid:>6}  {c.dst_addr:<44} {c.dst_port:>5}  {domain}", flush=True)
    except grpc.RpcError as e:
        print(f"\nDisconnected: {e.code().name} — {e.details()}", flush=True)
    except KeyboardInterrupt:
        print("\nStopped.", flush=True)

if __name__ == "__main__":
    main()
