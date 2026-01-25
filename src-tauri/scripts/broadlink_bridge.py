#!/usr/bin/env python3
"""
Broadlink Bridge Script

This script provides a command-line interface for interacting with Broadlink
IR/RF devices. It's designed to be called from the Tauri application.

Commands:
    discover [timeout]     - Discover devices on the network
    learn host mac devtype [ir|rf] - Enter learning mode
    send host mac devtype code     - Send an IR/RF code
    test host mac devtype          - Test if device is online

All output is JSON formatted for easy parsing.
"""

import sys
import json
import time
import socket

try:
    import broadlink
except ImportError:
    print(json.dumps({"error": "broadlink module not installed. Run: pip install broadlink"}))
    sys.exit(1)


def mac_str_to_bytes(mac_str: str) -> bytes:
    """Convert MAC address string to bytes."""
    mac_str = mac_str.replace(':', '').replace('-', '')
    return bytes.fromhex(mac_str)


def discover(timeout: int = 5):
    """Discover Broadlink devices on the network."""
    try:
        devices = broadlink.discover(timeout=timeout)
        result = []

        for device in devices:
            try:
                device.auth()
                result.append({
                    "type": hex(device.devtype),
                    "model": getattr(device, 'model', 'Unknown'),
                    "host": device.host[0],
                    "mac": ':'.join(format(x, '02x') for x in device.mac),
                    "name": getattr(device, 'name', None) or getattr(device, 'model', 'Broadlink Device')
                })
            except Exception as e:
                # Skip devices that fail auth
                continue

        print(json.dumps(result))
    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)


def learn(host: str, mac: str, devtype: str, signal_type: str = 'ir'):
    """Enter learning mode and wait for signal."""
    try:
        # Create device instance
        devtype_int = int(devtype, 16) if devtype.startswith('0x') else int(devtype)
        mac_bytes = mac_str_to_bytes(mac)
        device = broadlink.gendevice(devtype_int, (host, 80), mac_bytes)

        # Authenticate
        device.auth()

        if signal_type.lower() == 'rf':
            # RF learning is more complex - sweep for frequency first
            device.sweep_frequency()

            # Wait for frequency to be found
            found = False
            for _ in range(30):  # 30 second timeout
                time.sleep(1)
                if device.check_frequency():
                    found = True
                    break

            if not found:
                print(json.dumps({"error": "No RF frequency found"}))
                device.cancel_sweep_frequency()
                sys.exit(1)

            # Now find the RF packet
            device.find_rf_packet()

            # Wait for packet
            for _ in range(30):
                time.sleep(1)
                try:
                    data = device.check_data()
                    if data:
                        print(json.dumps({"code": data.hex()}))
                        return
                except (broadlink.exceptions.StorageError, broadlink.exceptions.ReadError):
                    continue

            print(json.dumps({"error": "RF learning timeout"}))
            sys.exit(1)

        else:
            # IR learning
            device.enter_learning()

            # Poll for learned data
            for _ in range(30):  # 30 second timeout
                time.sleep(1)
                try:
                    data = device.check_data()
                    if data:
                        print(json.dumps({"code": data.hex()}))
                        return
                except (broadlink.exceptions.StorageError, broadlink.exceptions.ReadError):
                    # No data yet
                    continue

            print(json.dumps({"error": "IR learning timeout"}))
            sys.exit(1)

    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)


def send(host: str, mac: str, devtype: str, code: str):
    """Send an IR/RF code to the device."""
    try:
        # Create device instance
        devtype_int = int(devtype, 16) if devtype.startswith('0x') else int(devtype)
        mac_bytes = mac_str_to_bytes(mac)
        device = broadlink.gendevice(devtype_int, (host, 80), mac_bytes)

        # Authenticate
        device.auth()

        # Send the code
        code_bytes = bytes.fromhex(code)
        device.send_data(code_bytes)

        print(json.dumps({"success": True}))

    except Exception as e:
        print(json.dumps({"success": False, "error": str(e)}))
        sys.exit(1)


def test(host: str, mac: str, devtype: str):
    """Test if device is online and responsive."""
    try:
        # Create device instance
        devtype_int = int(devtype, 16) if devtype.startswith('0x') else int(devtype)
        mac_bytes = mac_str_to_bytes(mac)
        device = broadlink.gendevice(devtype_int, (host, 80), mac_bytes)

        # Set a short timeout
        device.timeout = 5

        # Try to authenticate
        device.auth()

        print(json.dumps({"online": True}))

    except socket.timeout:
        print(json.dumps({"online": False}))
    except Exception as e:
        print(json.dumps({"online": False, "error": str(e)}))


def main():
    if len(sys.argv) < 2:
        print(json.dumps({"error": "Usage: broadlink_bridge.py <command> [args...]"}))
        sys.exit(1)

    command = sys.argv[1].lower()

    if command == 'discover':
        timeout = int(sys.argv[2]) if len(sys.argv) > 2 else 5
        discover(timeout)

    elif command == 'learn':
        if len(sys.argv) < 5:
            print(json.dumps({"error": "Usage: learn <host> <mac> <devtype> [ir|rf]"}))
            sys.exit(1)
        host = sys.argv[2]
        mac = sys.argv[3]
        devtype = sys.argv[4]
        signal_type = sys.argv[5] if len(sys.argv) > 5 else 'ir'
        learn(host, mac, devtype, signal_type)

    elif command == 'send':
        if len(sys.argv) < 6:
            print(json.dumps({"error": "Usage: send <host> <mac> <devtype> <code>"}))
            sys.exit(1)
        host = sys.argv[2]
        mac = sys.argv[3]
        devtype = sys.argv[4]
        code = sys.argv[5]
        send(host, mac, devtype, code)

    elif command == 'test':
        if len(sys.argv) < 5:
            print(json.dumps({"error": "Usage: test <host> <mac> <devtype>"}))
            sys.exit(1)
        host = sys.argv[2]
        mac = sys.argv[3]
        devtype = sys.argv[4]
        test(host, mac, devtype)

    else:
        print(json.dumps({"error": f"Unknown command: {command}"}))
        sys.exit(1)


if __name__ == '__main__':
    main()
