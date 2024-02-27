// import { randomBytes } from 'crypto';
import { randomBytes } from 'crypto';
import { readFileSync } from 'fs';
import path from 'path';
import { Address, Contract, Keypair, Operation, StrKey, hash, xdr } from 'stellar-sdk';
import { fileURLToPath } from 'url';
import { AddressBook } from './address_book.js';
import { config } from './env_config.js';
import { createTxBuilder, invoke, invokeTransaction } from './tx.js';

// Relative paths from __dirname
const CONTRACT_REL_PATH: object = {
  pair: '../../contracts/pair/target/wasm32-unknown-unknown/release/soroswap_pair.optimized.wasm',
  factory:
    '../../contracts/factory/target/wasm32-unknown-unknown/release/soroswap_factory.optimized.wasm',
  router: '../../contracts/router/target/wasm32-unknown-unknown/release/soroswap_router.optimized.wasm',
  token: '../../contracts/token/target/wasm32-unknown-unknown/release/soroban_token_contract.optimized.wasm',
};

const network = process.argv[2];
const loadedConfig = config(network);

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export async function installContract(wasmKey: string, addressBook: AddressBook, source: Keypair) {
  const contractWasm = readFileSync(
    path.join(__dirname, CONTRACT_REL_PATH[wasmKey as keyof object])
  );
  const wasmHash = hash(contractWasm);
  addressBook.setWasmHash(wasmKey, wasmHash.toString('hex'));
  console.log('Installing:', wasmKey, wasmHash.toString('hex'));
  const op = Operation.invokeHostFunction({
    func: xdr.HostFunction.hostFunctionTypeUploadContractWasm(contractWasm),
    auth: [],
  });
  addressBook.writeToFile();
  await invoke(op, source, false);
}

export async function deployContract(
  contractKey: string,
  wasmKey: string,
  addressBook: AddressBook,
  source: Keypair
) {
  const contractIdSalt = randomBytes(32);
  const networkId = hash(Buffer.from(loadedConfig.passphrase));
  const contractIdPreimage = xdr.ContractIdPreimage.contractIdPreimageFromAddress(
    new xdr.ContractIdPreimageFromAddress({
      address: Address.fromString(source.publicKey()).toScAddress(),
      salt: contractIdSalt,
    })
  );

  const hashIdPreimage = xdr.HashIdPreimage.envelopeTypeContractId(
    new xdr.HashIdPreimageContractId({
      networkId: networkId,
      contractIdPreimage: contractIdPreimage,
    })
  );
  console.log('Deploying WASM', wasmKey, 'for', contractKey);
  const contractId = StrKey.encodeContract(hash(hashIdPreimage.toXDR()));
  addressBook.setContractId(contractKey, contractId);
  const wasmHash = Buffer.from(addressBook.getWasmHash(wasmKey), 'hex');

  const deployFunction = xdr.HostFunction.hostFunctionTypeCreateContract(
    new xdr.CreateContractArgs({
      contractIdPreimage: contractIdPreimage,
      executable: xdr.ContractExecutable.contractExecutableWasm(wasmHash),
    })
  );

  addressBook.writeToFile();
  await invoke(
    Operation.invokeHostFunction({
      func: deployFunction,
      auth: [],
    }),
    loadedConfig.admin,
    false
  );
}

export async function invokeContract(
  contractKey: string,
  addressBook: AddressBook,
  method: string,
  params: xdr.ScVal[],
  source: Keypair
) {
  console.log("Invoking contract: ", contractKey, " with method: ", method);
  const contractAddress = addressBook.getContractId(contractKey);
  const contractInstance = new Contract(contractAddress);

  const contractOperation = contractInstance.call(method, ...params);
  await invoke(
    contractOperation,
    source,
    false,
  );  
}

// export async function deployStellarAsset(asset: Asset, addressBook: AddressBook, source: Keypair) {
//   const xdrAsset = asset.toXDRObject();
//   const networkId = hash(Buffer.from(config.passphrase));
//   const preimage = xdr.HashIdPreimage.envelopeTypeContractId(
//     new xdr.HashIdPreimageContractId({
//       networkId: networkId,
//       contractIdPreimage: xdr.ContractIdPreimage.contractIdPreimageFromAsset(xdrAsset),
//     })
//   );
//   const contractId = StrKey.encodeContract(hash(preimage.toXDR()));

//   addressBook.setContractId(asset.code, contractId);
//   const deployFunction = xdr.HostFunction.hostFunctionTypeCreateContract(
//     new xdr.CreateContractArgs({
//       contractIdPreimage: xdr.ContractIdPreimage.contractIdPreimageFromAsset(xdrAsset),
//       executable: xdr.ContractExecutable.contractExecutableStellarAsset(),
//     })
//   );
//   await invokeAndUnwrap(
//     Operation.invokeHostFunction({
//       func: deployFunction,
//       auth: [],
//     }),
//     source,
//     () => undefined
//   );
// }

export async function bumpContractInstance(
  contractKey: string,
  addressBook: AddressBook,
  source: Keypair
) {
  const address = Address.fromString(addressBook.getContractId(contractKey));
  console.log('bumping contract instance: ', address.toString());
  const contractInstanceXDR = xdr.LedgerKey.contractData(
    new xdr.LedgerKeyContractData({
      contract: address.toScAddress(),
      key: xdr.ScVal.scvLedgerKeyContractInstance(),
      durability: xdr.ContractDataDurability.persistent(),
    })
  );
  const bumpTransactionData = new xdr.SorobanTransactionData({
    resources: new xdr.SorobanResources({
      footprint: new xdr.LedgerFootprint({
        readOnly: [contractInstanceXDR],
        readWrite: [],
      }),
      instructions: 0,
      readBytes: 0,
      writeBytes: 0,
    }),
    resourceFee: xdr.Int64.fromString('0'),
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    ext: new xdr.ExtensionPoint(0),
  });

  const txBuilder = await createTxBuilder(source);
  txBuilder.addOperation(Operation.extendFootprintTtl({ extendTo: 535670 })); // 1 year
  txBuilder.setSorobanData(bumpTransactionData);
  const result = await invokeTransaction(txBuilder.build(), source, false);
  // @ts-ignore
  console.log(result.status, '\n');
}

export async function bumpContractCode(wasmKey: string, addressBook: AddressBook, source: Keypair) {
  console.log('bumping contract code: ', wasmKey);
  const wasmHash = Buffer.from(addressBook.getWasmHash(wasmKey), 'hex');
  const contractCodeXDR = xdr.LedgerKey.contractCode(
    new xdr.LedgerKeyContractCode({
      hash: wasmHash,
    })
  );
  const bumpTransactionData = new xdr.SorobanTransactionData({
    resources: new xdr.SorobanResources({
      footprint: new xdr.LedgerFootprint({
        readOnly: [contractCodeXDR],
        readWrite: [],
      }),
      instructions: 0,
      readBytes: 0,
      writeBytes: 0,
    }),
    resourceFee: xdr.Int64.fromString('0'),
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    ext: new xdr.ExtensionPoint(0),
  });

  const txBuilder = await createTxBuilder(source);
  txBuilder.addOperation(Operation.extendFootprintTtl({ extendTo: 535670 })); // 1 year
  txBuilder.setSorobanData(bumpTransactionData);
  const result = await invokeTransaction(txBuilder.build(), source, false);
  // @ts-ignore
  console.log(result.status, '\n');
}

export async function airdropAccount(user: Keypair) {
  try {
    console.log('Start funding');
    await loadedConfig.rpc.requestAirdrop(user.publicKey(), loadedConfig.friendbot);
    console.log('Funded: ', user.publicKey());
  } catch (e) {
    console.log(user.publicKey(), ' already funded');
  }
}

export async function deploySorobanToken(
  wasmKey: string,
  addressBook: AddressBook,
  source: Keypair
) {
  const contractIdSalt = randomBytes(32);
  const networkId = hash(Buffer.from(loadedConfig.passphrase));
  const contractIdPreimage = xdr.ContractIdPreimage.contractIdPreimageFromAddress(
    new xdr.ContractIdPreimageFromAddress({
      address: Address.fromString(source.publicKey()).toScAddress(),
      salt: contractIdSalt,
    })
  );

  const hashIdPreimage = xdr.HashIdPreimage.envelopeTypeContractId(
    new xdr.HashIdPreimageContractId({
      networkId: networkId,
      contractIdPreimage: contractIdPreimage,
    })
  );
  const contractId = StrKey.encodeContract(hash(hashIdPreimage.toXDR()));
  const wasmHash = Buffer.from(addressBook.getWasmHash(wasmKey), 'hex');

  const deployFunction = xdr.HostFunction.hostFunctionTypeCreateContract(
    new xdr.CreateContractArgs({
      contractIdPreimage: contractIdPreimage,
      executable: xdr.ContractExecutable.contractExecutableWasm(wasmHash),
    })
  );

  // addressBook.writeToFile();
  const result = await invoke(
    Operation.invokeHostFunction({
      func: deployFunction,
      auth: [],
    }),
    loadedConfig.admin,
    false
  );

  if (result) {
    return contractId;
  }
}