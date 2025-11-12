# Stratum V2 Implementation Validation

## Issues Identified

After reviewing the Stratum V2 implementation, several potential issues have been identified that need validation against the official specification:

### 1. Connection Architecture ⚠️

**Current Implementation**:
- Stratum V2 client uses `NetworkManager::send_to_peer()` which routes through Bitcoin P2P peer connections
- Assumes Stratum V2 can share connections with Bitcoin P2P protocol

**Potential Issue**:
- Stratum V2 typically uses **dedicated connections on separate ports** (e.g., port 3333 for Stratum V2 vs 8333 for Bitcoin P2P)
- Stratum V2 is a **different protocol** than Bitcoin P2P, not a sub-protocol
- May need **separate connection handling** instead of routing through NetworkManager's peer system

**Validation Needed**:
- Verify if Stratum V2 requires separate connections or can share transport
- Check if using transport abstraction is correct or if we need dedicated Stratum V2 connection management

### 2. Message Serialization Format ⚠️

**Current Implementation**:
- Uses JSON serialization (`serde_json::to_vec`) for message payloads
- TLV wrapper: Tag (u16) + Length (u32) + JSON payload

**Potential Issue**:
- Stratum V2 spec mentions "efficient binary protocol" and "50-66% bandwidth savings"
- JSON encoding may not match specification - spec may require binary/compact encoding
- Need to verify actual Stratum V2 message format

**Validation Needed**:
- Check Stratum V2 specification for actual serialization format
- Verify if JSON is correct or if binary encoding is required

### 3. TLV Encoding Format ⚠️

**Current Implementation**:
- Format: [4-byte length prefix][2-byte tag][4-byte length][payload]
- Little-endian encoding

**Potential Issue**:
- Stratum V2 spec may use different TLV format
- Endianness may differ
- Length prefix structure may differ

**Validation Needed**:
- Verify exact TLV format from Stratum V2 specification
- Check endianness requirements
- Verify message framing format

### 4. Connection Lifecycle ⚠️

**Current Implementation**:
- Client connects via `send_to_peer()` which assumes peer already exists
- No explicit connection establishment for Stratum V2
- Uses Bitcoin P2P peer management

**Potential Issue**:
- Stratum V2 requires **explicit connection establishment** (separate from Bitcoin P2P)
- May need to use transport directly (TcpTransport/IrohTransport) instead of NetworkManager
- Connection lifecycle should be managed separately

**Validation Needed**:
- Determine if Stratum V2 connections should use transport directly
- Check if separate connection pool is needed

### 5. Response Handling ⚠️

**Current Implementation**:
- Messages sent but responses not properly routed
- TODO comments indicate async message routing needed
- No request/response correlation

**Potential Issue**:
- Stratum V2 requires synchronous request/response handling
- Need proper message routing to match requests with responses
- Missing async message routing system

**Validation Needed**:
- Implement proper request/response correlation
- Add async message routing for responses

## Recommended Actions

### High Priority

1. **Verify Specification**: Review official Stratum V2 specification at https://stratumprotocol.org/
   - Check message serialization format (JSON vs binary)
   - Verify TLV encoding format
   - Confirm connection architecture

2. **Connection Architecture Decision**:
   - Option A: Separate Stratum V2 connections (dedicated ports, separate lifecycle)
   - Option B: Share transport abstraction (current approach - validate if correct)

3. **Message Format Validation**:
   - Verify if JSON encoding is correct
   - If binary encoding needed, implement binary serialization
   - Validate TLV format matches spec

### Medium Priority

4. **Implement Response Routing**:
   - Add request/response correlation
   - Implement async message routing system
   - Handle Setup Connection, Open Channel responses properly

5. **Connection Management**:
   - If separate connections needed, implement dedicated Stratum V2 connection pool
   - Manage connection lifecycle independently from Bitcoin P2P

## Current Status

**What's Working**:
- ✅ Module structure is sound
- ✅ Transport abstraction pattern is correct (works with TCP and Iroh)
- ✅ Message types are defined
- ✅ TLV encoding structure is in place
- ✅ Compiles successfully

**What Needs Validation**:
- ⚠️ Message serialization format (JSON vs binary)
- ⚠️ TLV encoding format (matches spec?)
- ⚠️ Connection architecture (separate vs shared)
- ⚠️ Response routing (async message handling)

## Next Steps

1. Review official Stratum V2 specification
2. Validate TLV and message encoding format
3. Decide on connection architecture (separate vs shared)
4. Implement proper response routing
5. Add integration tests with actual Stratum V2 pools (when available)

