import {
  Connection,
  Keypair,
  Ed25519Keypair,
  Ed25519Program,
  Transaction,
  sendAndConfirmTransaction,
  PublicKey,
  Commitment,
  AddressLookupTableProgram,
  TransactionInstruction,
  sendAndConfirmRawTransaction,
  ComputeBudgetProgram,
  GetVersionedTransactionConfig,
  SystemProgram,
  SystemInstruction,
} from "@solana/web3.js";
import fs from "fs";
import os from "os";
import BN from "bn.js";
import {
  getAssociatedTokenAddressSync,
  NATIVE_MINT,
  createAssociatedTokenAccountIdempotent,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  syncNative,
} from "@solana/spl-token";
import { publicKey, struct, u64, u8 } from "@project-serum/borsh";

const keyPairPath = os.homedir() + "/.config/solana/G9.json";
const PrivateKey = JSON.parse(fs.readFileSync(keyPairPath, "utf-8"));
let privateKey = Uint8Array.from(PrivateKey);
const wallet = Keypair.fromSecretKey(privateKey);
const walletPublicKey = wallet.publicKey;
const funding = new PublicKey("HkKy3jbbsD8strMpY3g6PFo3omsEKC4YmDBhtt994fZ4");
const solToFind = new PublicKey("HTGLeTmVsqRArne5HiF89kEz6zMCUG2u8of89pih5jCi");
const commitment: Commitment = "processed";
// const connection = new Connection("https://rpc-mainnet-fork.dappio.xyz", {
//   commitment,
//   wsEndpoint: "wss://rpc-mainnet-fork.dappio.xyz/ws",
// });
const connection = new Connection("http://127.0.0.1:8899", {
  wsEndpoint: "ws://localhost:8900/",
  commitment,
  confirmTransactionInitialTimeout: 1000000,
});
const ESCROW_PROGRAM_ID = new PublicKey(
  "GGJNxHtBwdQTYaz8yhmjCNy8NU8ayJB5GjYbDLkzSsuF"
);
const INIT_LAYOUT = struct([
  u8("instruction"),
  u64("amountToTrade"),
  u64("depositAmount"),
  u64("slot"),
]);
const EXCHANGE_LAYOUT = struct([u8("instruction"), u64("amountToTrade")]);
const U64_LAYOUT = struct([u64("u64")]);
const ESCROW_STATE_LAYOUT = struct([
  u8("isInitialized"),
  publicKey("initializerPubkey"),
  publicKey("mintA"),
  publicKey("mintB"),
  u64("expectedAmount"),
  u8("bump"),
  u64("seed"),
]);

async function init() {
  let airdrop = await connection.requestAirdrop(walletPublicKey, 100000000000);
  console.log("Airdrop: ", airdrop);
  let tokenAATa = await createAssociatedTokenAccountIdempotent(
    connection,
    wallet,
    NATIVE_MINT,
    walletPublicKey
  );
  let transfer = SystemProgram.transfer({
    toPubkey: tokenAATa,
    lamports: 100000000,
    fromPubkey: walletPublicKey,
  });
  let initTx = new Transaction().add(transfer);
  let initTxSig = await sendAndConfirmTransaction(
    connection,
    initTx,
    [wallet],
    {
      skipPreflight: true,
    }
  );

  let sync = await syncNative(connection, wallet, tokenAATa);
  let payload = Buffer.alloc(INIT_LAYOUT.span);
  let slot = await connection.getSlot();
  INIT_LAYOUT.encode(
    {
      instruction: new BN(0),
      amountToTrade: new BN(1000),
      depositAmount: new BN(100000000),
      slot: new BN(slot),
    },
    payload
  );

  let slotSeed = Buffer.alloc(8);
  U64_LAYOUT.encode({ u64: new BN(slot) }, slotSeed);
  let [pda, bump] = PublicKey.findProgramAddressSync(
    [slotSeed, walletPublicKey.toBuffer()],
    ESCROW_PROGRAM_ID
  );
  let vaultAta = getAssociatedTokenAddressSync(NATIVE_MINT, pda, true);
  let initEscrowIx = new TransactionInstruction({
    keys: [
      { pubkey: walletPublicKey, isSigner: true, isWritable: true },
      { pubkey: pda, isSigner: false, isWritable: true },
      { pubkey: vaultAta, isSigner: false, isWritable: true },
      { pubkey: tokenAATa, isSigner: false, isWritable: true },
      { pubkey: NATIVE_MINT, isSigner: false, isWritable: false },
      { pubkey: NATIVE_MINT, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      {
        pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: ESCROW_PROGRAM_ID,
    data: payload,
  });
  let tx = new Transaction().add(initEscrowIx);

  let txSig = await sendAndConfirmTransaction(connection, tx, [wallet], {
    skipPreflight: true,
    commitment: "confirmed",
  });
  console.log("init: " + txSig);
  return pda;
}
let result = init();

let exchangeResult = exchange();
async function exchange() {
  let taker = new Keypair();
  let airdrop = await connection.requestAirdrop(taker.publicKey, 100000000000);
  let _ = await result;
  let escrowKey = _;
  let len = (await connection.getAccountInfo(escrowKey))?.data.length;

  let escrowState = ESCROW_STATE_LAYOUT.decode(
    (await connection.getAccountInfo(escrowKey))?.data
  );
  let mintA = new PublicKey(escrowState.mintA);
  let mintB = new PublicKey(escrowState.mintB);

  let initializerPubkey = new PublicKey(escrowState.initializerPubkey);
  let mintBPayerAta = await createAssociatedTokenAccountIdempotent(
    connection,
    taker,
    mintB,
    taker.publicKey
  );
  let transfer = SystemProgram.transfer({
    toPubkey: mintBPayerAta,
    lamports: 100000000,
    fromPubkey: walletPublicKey,
  });
  let initTx = new Transaction().add(transfer);
  let initTxSig = await sendAndConfirmTransaction(
    connection,
    initTx,
    [wallet],
    {
      skipPreflight: true,
    }
  );

  let sync = await syncNative(connection, wallet, mintBPayerAta);
  let mintAReceiver = getAssociatedTokenAddressSync(mintA, taker.publicKey);
  let mintBRecieveAta = getAssociatedTokenAddressSync(mintB, initializerPubkey);

  let payload = Buffer.alloc(EXCHANGE_LAYOUT.span);
  EXCHANGE_LAYOUT.encode(
    {
      instruction: new BN(1),
      amountToTrade: escrowState.expectedAmount,
    },
    payload
  );

  let vaultAta = getAssociatedTokenAddressSync(
    escrowState.mintA,
    escrowKey,
    true
  );
  let ix = new TransactionInstruction({
    keys: [
      { pubkey: taker.publicKey, isSigner: true, isWritable: true },
      {
        pubkey: initializerPubkey,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: escrowKey, isSigner: false, isWritable: true },
      { pubkey: vaultAta, isSigner: false, isWritable: true },
      { pubkey: mintAReceiver, isSigner: false, isWritable: true },
      { pubkey: mintBRecieveAta, isSigner: false, isWritable: true },

      { pubkey: mintBPayerAta, isSigner: false, isWritable: true },

      { pubkey: mintA, isSigner: false, isWritable: false },
      { pubkey: mintB, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      {
        pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: payload,
    programId: ESCROW_PROGRAM_ID,
  });

  let tx = new Transaction().add(ix);

  let txSig = await sendAndConfirmTransaction(connection, tx, [taker], {
    skipPreflight: true,
    commitment: "confirmed",
  });
  console.log("exhcange: " + txSig);
}
