import { ApiPromise, WsProvider } from "@polkadot/api";
import 'dotenv/config';

const WS_ENDPOINT = process.env.WS_ENDPOINT;

console.log("Connecting to blockchain...");
const wsProvider = new WsProvider(WS_ENDPOINT);
const api = await ApiPromise.create({ provider: wsProvider });

/// The header has no block number because Xode chain is using 
/// a custom header type that does not include a block number field.

let headerBlockHash = await api.rpc.chain.getFinalizedHead();
let block = await api.rpc.chain.getBlock(headerBlockHash);

const parentHash = block.block.header.parentHash;
const parentHeader = await api.rpc.chain.getHeader(parentHash);
const blockNumber = parentHeader.number.toNumber() + 1;

/// console.log("Latest Block Hash:", headerBlockHash.toHex());
console.log("Finalized Block Number:", blockNumber);

/// Other option in getting the latest finalized block number.
/// Because system.number is a runtime storage item, not dependent 
/// on header structure.

const finalizedHash = await api.rpc.chain.getFinalizedHead();
const bn = await api.query.system.number.at(finalizedHash);

console.log("Finalized Block number:", bn.toNumber());

process.exit(0);