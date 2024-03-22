/* eslint-disable @typescript-eslint/no-var-requires */

// Run with command:
// yarn run-example ./evm/send-evm-transaction.ts

// In this example we will send an ERC-20 token transfer using a SecretManager.
import { strictEqual } from 'assert/strict';
import { Buffer } from 'buffer';
import Web3 from 'web3';
import { Common } from '@ethereumjs/common';
import { Transaction, TxData } from '@ethereumjs/tx';
import { RLP } from '@ethereumjs/rlp';
import { fromRpcSig } from '@ethereumjs/util';

import { SecretManager } from '@iota/sdk';
import { Bech32 } from '@iota/crypto.js';

import { ISC_SANDBOX_ABI } from './isc-sandbox.abi';

// This example uses secrets in environment variables for simplicity which should not be done in production.
require('dotenv').config({ path: '.env' });

const RPC_ENDPOINT = 'https://json-rpc.evm.testnet.shimmer.network';
const CHAIN_ID = 1073;
const ETHEREUM_COIN_TYPE = 60;
const MAGIC_CONTRACT = '0x1074000000000000000000000000000000000000';
const L1_RECIPIENT =
    'rms1qzzk86qv30l4e85ljtccxa0ruy8y7u8zn2dle3g8dv2tl2m4cu227a7n2wj';

const TX_OPTIONS = {
    common: Common.custom({
        chainId: CHAIN_ID,
    }),
    freeze: false,
};
const WEI_PER_GLOW = BigInt(1_000_000_000_000);

const SEND_METADATA = {
    targetContract: 0,
    entrypoint: 0,
    params: {
        items: [],
    },
    allowance: {
        baseTokens: '0',
        nativeTokens: [],
        nfts: [],
    },
    gasBudget: 0,
};

const SEND_OPTIONS = {
    timelock: 0,
    expiration: {
        time: 0,
        returnAddress: {
            data: new Uint8Array([]),
        },
    },
};

async function run(): Promise<void> {
    const provider = new Web3(RPC_ENDPOINT);
    for (const envVar of ['MNEMONIC']) {
        if (!(envVar in process.env)) {
            throw new Error(`.env ${envVar} is undefined, see .env.example`);
        }
    }
    try {
        const mnemonicSecretManager = {
            mnemonic: process.env.MNEMONIC as string,
        };

        const secretManager = new SecretManager(mnemonicSecretManager);

        const addresses = await secretManager.generateEvmAddresses({
            coinType: ETHEREUM_COIN_TYPE,
            accountIndex: 0,
        });
        const senderAddress = addresses[0];
        console.log('sender address', senderAddress);
        // 1. Create unsigned transaction data
        const txData = await createTxData(provider, senderAddress);
        const transaction = Transaction.fromTxData(txData, TX_OPTIONS);

        // 2. Create messageToSign by external signer
        const message = transaction.getMessageToSign(false);
        const serializedMessage = Buffer.from(RLP.encode(message));
        const messageToSign = '0x' + serializedMessage.toString('hex');

        // 3. Sign message with external signer
        const bip44Path = {
            coinType: ETHEREUM_COIN_TYPE,
            account: 0,
            change: 0,
            addressIndex: 0,
        };
        const { signature } = await secretManager.signSecp256k1Ecdsa(
            messageToSign,
            bip44Path,
        );

        // 4. Make Secp256k1Ecdsa an Eip155Compatible Signature
        const ecdsaSignature = fromRpcSig(signature);
        ecdsaSignature.v = convertsVToEip155Compatible(
            ecdsaSignature.v,
            CHAIN_ID,
        );

        // 5. Sign Transaction
        const signedTransaction = createSignedTransaction(
            transaction,
            ecdsaSignature,
        );

        // Testing: check sender address matches
        strictEqual(
            senderAddress,
            signedTransaction.getSenderAddress().toString(),
            'Mismatch in addresses',
        );

        // 6. Send signed transaction
        const hexSignedTransaction =
            getHexEncodedTransaction(signedTransaction);
        const sentTransaction = await provider.eth.sendSignedTransaction(
            hexSignedTransaction,
        );
        console.log('sent Transaction', sentTransaction);
    } catch (error) {
        console.error('Error: ', error);
    }
    process.exit(0);
}

function createSignedTransaction(
    transaction: Transaction,
    signature: any,
): Transaction {
    const rawTx = transaction.raw();

    const vHex = padHexString(signature.v.toString(16));
    rawTx[6] = Buffer.from(vHex, 'hex');
    rawTx[7] = signature.r;
    rawTx[8] = signature.s;
    const signedTransaction = Transaction.fromValuesArray(rawTx, TX_OPTIONS);

    return signedTransaction;
}

function getHexEncodedTransaction(transaction: Transaction): string {
    const serializedTransaction = transaction.serialize();
    const hexEncodedTransaction = '0x' + serializedTransaction.toString('hex');
    return hexEncodedTransaction;
}

function convertsVToEip155Compatible(v: bigint, chainId: number): bigint {
    const parity = Number(v) % 27;
    const newV = chainId * 2 + (35 + parity);
    return BigInt(newV);
}

async function createTxData(provider: Web3, address: string): Promise<TxData> {
    const magicContract = new provider.eth.Contract(
        ISC_SANDBOX_ABI,
        MAGIC_CONTRACT,
    );

    const balanceInWei = await provider.eth.getBalance(address);
    const balance = (BigInt(balanceInWei) / WEI_PER_GLOW).toString();

    const nonce = provider.utils.toHex(
        await provider.eth.getTransactionCount(address),
    );

    const _gasPrice = await provider.eth.getGasPrice();
    const gasPrice = provider.utils.toHex(_gasPrice);

    const to = MAGIC_CONTRACT;
    const value = provider.utils.toHex('0');

    const recipient = buildLayer1RecipientAddress(L1_RECIPIENT);

    async function getDataAndGasLimit(baseTokens: string) {
        const assetAllowance = {
            baseTokens,
            nativeTokens: [],
            nfts: [],
        };

        const data = magicContract.methods
            .send(recipient, assetAllowance, false, SEND_METADATA, SEND_OPTIONS)
            .encodeABI();

        const estimatedGas = await provider.eth.estimateGas({
            from: address,
            to: MAGIC_CONTRACT,
            data,
            value: '0',
        });
        const gasLimit = provider.utils.toHex(estimatedGas);
        return { gasLimit, data };
    }

    // Initial Run
    const firstRun = await getDataAndGasLimit(balance);
    console.log('first run balance:', balance);
    console.log('first run gas:', BigInt(firstRun.gasLimit)); // Why is this amount lower than the estimation of the second run?

    // Second run -> using the data and gas limit of adjusted balance won't work
    let adjustedBalance = (
        BigInt(balance) - BigInt(firstRun.gasLimit)
    ).toString();
    const secondRun = await getDataAndGasLimit(adjustedBalance);

    console.log('second run balance:', adjustedBalance);
    console.log('second run gas:', BigInt(secondRun.gasLimit));

    // Third Run -> using this data and gas limit results in a successful transaction
    adjustedBalance = (BigInt(balance) - BigInt(secondRun.gasLimit)).toString();
    const thirdRun = await getDataAndGasLimit(adjustedBalance);

    console.log('third run balance:', adjustedBalance);
    console.log('third run gas:', BigInt(thirdRun.gasLimit));

    return {
        nonce,
        gasPrice,
        gasLimit: secondRun.gasLimit,
        to,
        value,
        data: secondRun.data,
    };
}

function padHexString(str: string): string {
    return str.length % 2 !== 0 ? '0' + str : str;
}

function buildLayer1RecipientAddress(address: string) {
    const { addressBytes } = fromBech32(address, 'rms') ?? {};
    return {
        data: new Uint8Array([0, ...(addressBytes ?? [])]),
    };
}

function fromBech32(
    bech32Text: string,
    humanReadablePart: string,
): {
    addressType: number;
    addressBytes: Uint8Array;
} {
    const decoded = Bech32.decode(bech32Text);
    if (decoded) {
        if (decoded.humanReadablePart !== humanReadablePart) {
            throw new Error(
                `The hrp part of the address should be ${humanReadablePart}, it is ${decoded.humanReadablePart}`,
            );
        }

        if (decoded.data.length === 0) {
            throw new Error(
                'The data part of the address should be at least length 1, it is 0',
            );
        }

        const addressType = decoded.data[0];
        const addressBytes = decoded.data.slice(1);

        return {
            addressType,
            addressBytes,
        };
    } else {
        throw new Error(`Bech32 decoding of ${bech32Text} failed!`);
    }
}

run();
