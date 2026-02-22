# API Reference

## RPC Endpoints

### Chain

```
chain_getBlock
chain_getBlockHash
chain_getFinalizedHead
chain_getHeader
```

### State

```
state_getStorage
state_getRuntimeVersion
state_queryStorage
```

### System

```
system_name
system_version
system_chain
```

## Usage Examples

### Get Block

```javascript
const { ApiPromise, WsProvider } = require('@polkadot/api');

const api = await ApiPromise.create({
  provider: new WsProvider('wss://rpc.xode.network')
});

const block = await api.rpc.chain.getBlock();
console.log(block.toHuman());
```

### Query Storage

```javascript
const balance = await api.query.system.account(address);
console.log(balance.free.toHuman());
```
