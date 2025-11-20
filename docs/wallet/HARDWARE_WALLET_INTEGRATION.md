# Hardware Wallet Integration Framework
## OpenSyria Digital Lira - Ledger & Trezor Support

**Version:** 1.0.0  
**Status:** Architecture Design  
**Target Release:** Phase 3 (Week 12)  
**Security Priority:** CRITICAL

---

## Overview

Hardware wallets provide the highest level of security for private key management by keeping keys isolated in a secure element that never leaves the device. This document outlines the architecture for integrating Ledger and Trezor hardware wallets with OpenSyria Digital Lira.

### Supported Devices

#### Tier 1: Immediate Support (Phase 3)
- **Ledger Nano S Plus** - Most popular, affordable
- **Ledger Nano X** - Bluetooth support, larger storage
- **Trezor Model T** - Touchscreen, open-source firmware

#### Tier 2: Future Support (Post-Mainnet)
- Trezor Safe 3
- Ledger Stax
- GridPlus Lattice1
- CoolWallet Pro

---

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    OpenSyria Wallet UI                      │
│                  (Desktop/Web/Mobile)                       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ USB / Bluetooth / WebUSB
                         │
┌────────────────────────┴────────────────────────────────────┐
│              Hardware Wallet Transport Layer                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Ledger USB   │  │ Trezor USB   │  │  WebUSB      │     │
│  │ Transport    │  │  Transport   │  │  Transport   │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ APDU Commands / Protobuf
                         │
┌────────────────────────┴────────────────────────────────────┐
│                  OpenSyria App on Device                    │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  - BIP-32 HD Key Derivation (m/44'/963'/0'/0)        │  │
│  │  - Ed25519 Signature Generation                      │  │
│  │  - Transaction Parsing & Display                     │  │
│  │  - User Confirmation on Device Screen                │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                         │
                         │ Cryptographic Operations
                         │
┌────────────────────────┴────────────────────────────────────┐
│                   Secure Element (SE)                       │
│  - Private keys never leave device                          │
│  - PIN/Passphrase protection                                │
│  - Anti-tampering protections                               │
└─────────────────────────────────────────────────────────────┘
```

---

## BIP-44 Derivation Path

OpenSyria uses a custom BIP-44 derivation path:

```
m / purpose' / coin_type' / account' / change / address_index

m / 44' / 963' / 0' / 0 / 0  → First receiving address
m / 44' / 963' / 0' / 1 / 0  → First change address
m / 44' / 963' / 1' / 0 / 0  → Second account
```

**Coin Type:** 963 (SLIP-44 registered for Syria)  
**Purpose:** 44 (BIP-44 compliant)

### Address Derivation

```rust
// crates/wallet/src/hardware_wallet.rs

use bip32::{DerivationPath, XPrv, Mnemonic};
use ed25519_dalek::{PublicKey, SecretKey};

pub struct HardwareWallet {
    device: Box<dyn HardwareDevice>,
    derivation_path: DerivationPath,
}

impl HardwareWallet {
    pub fn derive_address(&self, account: u32, index: u32) -> Result<String> {
        // Path: m/44'/963'/account'/0/index
        let path = format!("m/44'/963'/{:?}'/0/{:?}", account, index);
        let derivation_path = DerivationPath::from_str(&path)?;
        
        // Request public key from hardware device
        let pubkey = self.device.get_public_key(&derivation_path)?;
        
        // Convert to OpenSyria address format (Bech32)
        let address = self.pubkey_to_address(pubkey)?;
        Ok(address)
    }
    
    fn pubkey_to_address(&self, pubkey: PublicKey) -> Result<String> {
        // OpenSyria address format: syl1q[bech32 encoded pubkey hash]
        let pubkey_hash = blake2b_hash(pubkey.as_bytes());
        let address = bech32::encode("syl", pubkey_hash.to_vec(), bech32::Variant::Bech32)?;
        Ok(address)
    }
}
```

---

## Ledger Integration

### Ledger App Development

**Repository:** `opensyria/ledger-app-opensyria`

**Technology Stack:**
- Language: C (Ledger SDK)
- Build System: Make
- Testing: Speculos (Ledger emulator)

**App Structure:**
```
ledger-app-opensyria/
├── src/
│   ├── main.c              # Entry point, APDU dispatcher
│   ├── sign.c              # Transaction signing logic
│   ├── derive.c            # BIP-32 key derivation
│   ├── ui.c                # Screen rendering (Nano S/X)
│   └── crypto.c            # Ed25519 operations
├── glyphs/                 # Icons (OpenSyria logo)
├── Makefile
├── app.json                # Ledger app manifest
└── tests/
    └── test_app.py         # Integration tests
```

### APDU Commands

**Get Public Key (0x02):**
```c
// Request
CLA: 0xE0
INS: 0x02
P1: 0x00 (no display) / 0x01 (display on screen)
P2: 0x00
DATA: BIP-32 path (4 bytes per level)

// Response
DATA: 32-byte Ed25519 public key
```

**Sign Transaction (0x04):**
```c
// Request
CLA: 0xE0
INS: 0x04
P1: 0x00 (first chunk) / 0x80 (subsequent chunks)
P2: 0x00
DATA: Transaction data (max 255 bytes per chunk)

// Response
DATA: 64-byte Ed25519 signature
```

**Get App Version (0x01):**
```c
// Request
CLA: 0xE0
INS: 0x01
P1: 0x00
P2: 0x00

// Response
DATA: [major, minor, patch] (3 bytes)
```

### Transaction Display

Users must confirm transactions on the Ledger screen:

```
┌──────────────────┐
│  OpenSyria       │
│  Review Transfer │
├──────────────────┤
│ Amount:          │
│ 1,234.56 SYL     │
├──────────────────┤
│ To:              │
│ syl1q7f3...a9c2  │
├──────────────────┤
│ Fee:             │
│ 0.01 SYL         │
├──────────────────┤
│ ✓ Approve        │
│ ✗ Reject         │
└──────────────────┘
```

### Rust Client Library

```rust
// crates/wallet/src/ledger.rs

use ledger_transport_hid::{TransportNativeHID, hidapi::HidApi};
use ledger_apdu::{APDUCommand, APDUAnswer};

pub struct LedgerDevice {
    transport: TransportNativeHID,
}

impl LedgerDevice {
    pub fn connect() -> Result<Self> {
        let hidapi = HidApi::new()?;
        let transport = TransportNativeHID::new(&hidapi)?;
        Ok(Self { transport })
    }
    
    pub fn get_public_key(&self, path: &DerivationPath) -> Result<PublicKey> {
        let command = APDUCommand {
            cla: 0xE0,
            ins: 0x02,
            p1: 0x00,  // No display
            p2: 0x00,
            data: path.to_bytes(),
        };
        
        let response = self.transport.exchange(&command)?;
        
        if response.retcode() != 0x9000 {
            return Err(Error::LedgerError(response.retcode()));
        }
        
        let pubkey_bytes = response.data();
        let pubkey = PublicKey::from_bytes(pubkey_bytes)?;
        Ok(pubkey)
    }
    
    pub fn sign_transaction(&self, tx: &Transaction, path: &DerivationPath) -> Result<Signature> {
        // Serialize transaction for hardware wallet
        let tx_bytes = tx.serialize_for_signing()?;
        
        // Send transaction in chunks (max 255 bytes per APDU)
        let chunks = tx_bytes.chunks(255);
        let total_chunks = chunks.len();
        
        for (i, chunk) in chunks.enumerate() {
            let p1 = if i == 0 { 0x00 } else { 0x80 };  // First vs subsequent
            let p2 = if i == total_chunks - 1 { 0x00 } else { 0x80 };  // More data coming
            
            let command = APDUCommand {
                cla: 0xE0,
                ins: 0x04,
                p1,
                p2,
                data: chunk.to_vec(),
            };
            
            let response = self.transport.exchange(&command)?;
            
            // Only last chunk returns signature
            if i == total_chunks - 1 {
                if response.retcode() == 0x9000 {
                    let sig_bytes = response.data();
                    let signature = Signature::from_bytes(sig_bytes)?;
                    return Ok(signature);
                } else if response.retcode() == 0x6985 {
                    return Err(Error::UserRejected);
                } else {
                    return Err(Error::LedgerError(response.retcode()));
                }
            }
        }
        
        Err(Error::InvalidResponse)
    }
}
```

---

## Trezor Integration

### Trezor Protobuf Messages

**Repository:** Fork `trezor/trezor-firmware` and add OpenSyria support

**Protobuf Definitions (`messages-opensyria.proto`):**

```protobuf
syntax = "proto2";
package hw.trezor.messages.opensyria;

// Request: Get OpenSyria address
message OpenSyriaGetAddress {
    repeated uint32 address_n = 1;      // BIP-32 path
    optional bool show_display = 2;     // Show on screen
}

// Response: OpenSyria address
message OpenSyriaAddress {
    required string address = 1;         // Bech32 address (syl1q...)
}

// Request: Sign OpenSyria transaction
message OpenSyriaSignTx {
    repeated uint32 address_n = 1;      // BIP-32 path
    required uint64 nonce = 2;          // Transaction nonce
    required bytes to = 3;              // Recipient address (32 bytes)
    required uint64 amount = 4;         // Amount in smallest unit
    required uint64 fee = 5;            // Transaction fee
    optional uint32 chain_id = 6;       // Network ID (963 mainnet)
}

// Response: Transaction signature
message OpenSyriaTxSignature {
    required bytes signature = 1;       // Ed25519 signature (64 bytes)
}
```

### Rust Client Library

```rust
// crates/wallet/src/trezor.rs

use trezor_client::{TrezorClient, TransportNative, messages};

pub struct TrezorDevice {
    client: TrezorClient<TransportNative>,
}

impl TrezorDevice {
    pub fn connect() -> Result<Self> {
        let mut client = TrezorClient::new()?;
        
        // Initialize session
        client.init_device(None)?;
        
        Ok(Self { client })
    }
    
    pub fn get_address(&self, path: &DerivationPath, show_display: bool) -> Result<String> {
        let message = messages::opensyria::OpenSyriaGetAddress {
            address_n: path.to_u32_vec(),
            show_display: Some(show_display),
        };
        
        let response: messages::opensyria::OpenSyriaAddress = self.client.call(message)?;
        Ok(response.address)
    }
    
    pub fn sign_transaction(&self, tx: &Transaction, path: &DerivationPath) -> Result<Signature> {
        let message = messages::opensyria::OpenSyriaSignTx {
            address_n: path.to_u32_vec(),
            nonce: tx.nonce,
            to: tx.to.as_bytes().to_vec(),
            amount: tx.amount,
            fee: tx.fee,
            chain_id: Some(963),  // Mainnet
        };
        
        let response: messages::opensyria::OpenSyriaTxSignature = self.client.call(message)?;
        let signature = Signature::from_bytes(&response.signature)?;
        Ok(signature)
    }
}
```

---

## WebUSB Integration (Browser Support)

For web-based wallets, use WebUSB API:

```typescript
// frontend/src/hardware-wallet/ledger-web.ts

import TransportWebUSB from "@ledgerhq/hw-transport-webusb";

export class LedgerWebWallet {
    private transport: TransportWebUSB;
    
    async connect(): Promise<void> {
        this.transport = await TransportWebUSB.create();
    }
    
    async getAddress(account: number, index: number): Promise<string> {
        const path = `44'/963'/${account}'/0/${index}`;
        const pathBuffer = this.serializePath(path);
        
        const response = await this.transport.send(
            0xE0,  // CLA
            0x02,  // INS (Get Public Key)
            0x00,  // P1
            0x00,  // P2
            pathBuffer
        );
        
        const pubkey = response.slice(0, 32);
        return this.pubkeyToAddress(pubkey);
    }
    
    async signTransaction(tx: Transaction, account: number, index: number): Promise<string> {
        const path = `44'/963'/${account}'/0/${index}`;
        const txBytes = tx.serialize();
        
        // Send transaction data
        const response = await this.transport.send(
            0xE0,  // CLA
            0x04,  // INS (Sign Transaction)
            0x00,  // P1
            0x00,  // P2
            Buffer.concat([this.serializePath(path), txBytes])
        );
        
        const signature = response.slice(0, 64).toString('hex');
        return signature;
    }
    
    private serializePath(path: string): Buffer {
        const segments = path.split('/').map(s => {
            const hardened = s.endsWith("'");
            const value = parseInt(s.replace("'", ""));
            return hardened ? (value | 0x80000000) : value;
        });
        
        const buffer = Buffer.alloc(1 + segments.length * 4);
        buffer.writeUInt8(segments.length, 0);
        segments.forEach((segment, i) => {
            buffer.writeUInt32BE(segment, 1 + i * 4);
        });
        
        return buffer;
    }
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_derivation_path_parsing() {
        let path = DerivationPath::from_str("m/44'/963'/0'/0/0").unwrap();
        assert_eq!(path.to_string(), "m/44'/963'/0'/0/0");
    }
    
    #[test]
    fn test_address_generation() {
        let pubkey = PublicKey::from_bytes(&[0u8; 32]).unwrap();
        let address = HardwareWallet::pubkey_to_address(pubkey).unwrap();
        assert!(address.starts_with("syl1q"));
    }
    
    #[test]
    fn test_transaction_serialization() {
        let tx = Transaction {
            from: "syl1qalice...".to_string(),
            to: "syl1qbob...".to_string(),
            amount: 1000,
            nonce: 1,
            fee: 10,
            chain_id: 963,
        };
        
        let serialized = tx.serialize_for_signing().unwrap();
        assert!(serialized.len() > 0);
    }
}
```

### Integration Tests (Emulated Devices)

**Ledger Speculos Emulator:**
```bash
# Install Speculos
pip install speculos

# Run emulator with OpenSyria app
speculos --model nanos --display headless ledger-app-opensyria/bin/app.elf &

# Run integration tests
cargo test --test ledger_integration -- --nocapture
```

**Trezor Emulator:**
```bash
# Clone trezor-firmware
git clone https://github.com/trezor/trezor-firmware
cd trezor-firmware

# Build emulator
cd core
make vendor build_unix

# Run emulator
./build/unix/trezor-emu-core &

# Run integration tests
cargo test --test trezor_integration -- --nocapture
```

### Manual Testing Checklist

- [ ] Connect Ledger Nano S Plus via USB
- [ ] Install OpenSyria app from Ledger Live
- [ ] Derive first address (m/44'/963'/0'/0/0)
- [ ] Verify address matches expected value
- [ ] Create test transaction
- [ ] Confirm transaction details on device screen
- [ ] Approve transaction
- [ ] Verify signature is valid
- [ ] Reject transaction (test rejection handling)
- [ ] Test with multiple accounts (account 0, 1, 2)
- [ ] Test with Trezor Model T
- [ ] Test WebUSB in Chrome browser

---

## Security Considerations

### Threat Model

**Threats Mitigated:**
- ✅ Private key theft (keys never leave device)
- ✅ Malware on host computer (user confirms on device)
- ✅ MITM attacks (transaction shown on secure screen)
- ✅ Social engineering (user verifies recipient address)

**Remaining Risks:**
- ⚠️ Supply chain attacks (compromised hardware)
- ⚠️ Physical theft (PIN protects but not foolproof)
- ⚠️ $5 wrench attack (physical coercion)
- ⚠️ Firmware vulnerabilities (keep devices updated)

### Best Practices

**For Users:**
1. **Buy directly from manufacturer** (avoid resellers)
2. **Verify device authenticity** (check security seals)
3. **Use strong PIN** (8 digits minimum)
4. **Enable passphrase** (25th word for plausible deniability)
5. **Backup seed phrase securely** (metal backup, fireproof safe)
6. **Verify addresses on device screen** (never trust host computer)
7. **Keep firmware updated** (patch security vulnerabilities)

**For Developers:**
1. **Never request private keys** (only public keys and signatures)
2. **Display full transaction details** (amount, recipient, fee)
3. **Implement timeout for user confirmation** (prevent indefinite waiting)
4. **Handle user rejection gracefully** (don't retry automatically)
5. **Verify signatures before broadcasting** (sanity check)

---

## Wallet UI Integration

### React Component Example

```typescript
// frontend/src/components/HardwareWalletConnect.tsx

import React, { useState } from 'react';
import { LedgerWebWallet } from '../hardware-wallet/ledger-web';

export const HardwareWalletConnect: React.FC = () => {
    const [wallet, setWallet] = useState<LedgerWebWallet | null>(null);
    const [address, setAddress] = useState<string>('');
    const [connecting, setConnecting] = useState(false);
    
    const connectLedger = async () => {
        setConnecting(true);
        try {
            const ledger = new LedgerWebWallet();
            await ledger.connect();
            setWallet(ledger);
            
            // Get first address
            const addr = await ledger.getAddress(0, 0);
            setAddress(addr);
            
            alert(`Connected! Address: ${addr}`);
        } catch (error) {
            console.error('Failed to connect:', error);
            alert('Failed to connect to Ledger. Make sure device is unlocked and OpenSyria app is open.');
        } finally {
            setConnecting(false);
        }
    };
    
    return (
        <div className="hardware-wallet-connect">
            <h2>Connect Hardware Wallet</h2>
            
            {!wallet ? (
                <button onClick={connectLedger} disabled={connecting}>
                    {connecting ? 'Connecting...' : 'Connect Ledger'}
                </button>
            ) : (
                <div>
                    <p>✓ Connected</p>
                    <p>Address: {address}</p>
                </div>
            )}
            
            <div className="instructions">
                <h3>Instructions:</h3>
                <ol>
                    <li>Connect your Ledger device via USB</li>
                    <li>Unlock with PIN</li>
                    <li>Open the OpenSyria app</li>
                    <li>Click "Connect Ledger"</li>
                </ol>
            </div>
        </div>
    );
};
```

---

## Deployment Roadmap

### Phase 1: Development (Weeks 1-4)
- [ ] Implement BIP-32 derivation path support
- [ ] Create Ledger app (C code)
- [ ] Create Rust client library
- [ ] Implement WebUSB transport
- [ ] Unit tests for all components

### Phase 2: Testing (Weeks 5-6)
- [ ] Integration tests with Speculos emulator
- [ ] Manual testing on Ledger Nano S Plus
- [ ] Manual testing on Ledger Nano X
- [ ] Manual testing on Trezor Model T
- [ ] Security audit of hardware wallet integration

### Phase 3: Ledger Approval (Weeks 7-10)
- [ ] Submit app to Ledger for review
- [ ] Address Ledger security team feedback
- [ ] Pass Ledger certification process
- [ ] App published to Ledger Live

### Phase 4: Mainnet Launch (Week 11-12)
- [ ] Deploy wallet UI with hardware wallet support
- [ ] User documentation (guides, videos)
- [ ] Community testing period
- [ ] Mainnet launch announcement

---

## User Documentation

### Quick Start Guide

**Step 1: Install OpenSyria App on Ledger**
1. Open Ledger Live
2. Go to "Manager"
3. Search for "OpenSyria"
4. Click "Install"

**Step 2: Connect to OpenSyria Wallet**
1. Open OpenSyria wallet (web or desktop)
2. Click "Connect Hardware Wallet"
3. Select "Ledger" or "Trezor"
4. Unlock device and open OpenSyria app
5. Follow on-screen instructions

**Step 3: Send Transaction**
1. Enter recipient address
2. Enter amount
3. Click "Send"
4. Verify details on hardware wallet screen
5. Approve transaction on device
6. Wait for confirmation on blockchain

---

## Troubleshooting

### Common Issues

**"Device not found"**
- Ensure USB cable is connected
- Try different USB port
- On Linux: Add udev rules for Ledger/Trezor
- On Windows: Install device drivers

**"App not open"**
- Open OpenSyria app on device (not Bitcoin or Ethereum)
- Device must be unlocked (enter PIN)

**"User rejected transaction"**
- Check if you accidentally pressed "Reject" on device
- Verify transaction details match your intention
- Try again

**"Invalid signature"**
- Ensure using correct derivation path
- Check chain ID matches (963 for mainnet, 963000 for testnet)
- Verify transaction serialization format

---

## References

- **BIP-32 (HD Wallets):** https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki
- **BIP-44 (Multi-Account Hierarchy):** https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki
- **SLIP-44 (Coin Types):** https://github.com/satoshilabs/slips/blob/master/slip-0044.md
- **Ledger SDK:** https://github.com/LedgerHQ/ledger-app-builder
- **Trezor Firmware:** https://github.com/trezor/trezor-firmware
- **WebUSB API:** https://wicg.github.io/webusb/

---

**Document Owner:** Wallet Team  
**Last Updated:** November 19, 2025  
**Next Review:** Post-Phase 3 implementation

*"Your keys, your coins. Hardware wallets make it real."*  
*"مفاتيحك، عملاتك. محافظ الأجهزة تجعلها حقيقية"*
