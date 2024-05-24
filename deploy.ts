// const { SigningArchwayClient } = require("@archwayhq/arch3.js");
// const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
// const fs = require("fs");
// const { toByteArray } = require("base64-js");
// const dotenv = require("dotenv");

import { SigningArchwayClient } from "@archwayhq/arch3.js";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import fs from "fs";
import { toByteArray } from "base64-js";
import dotenv from "dotenv";

dotenv.config();

async function main() {
  const network = {
    chainId: "constantine-3",
    endpoint: "https://rpc.constantine.archway.io:443",
    prefix: "archway",
  };
  const mnemonic: string = process.env.MNEMONIC as string;
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: network.prefix,
  });
  const accounts = await wallet.getAccounts();

  const accountAddress = accounts[0].address;
  const beneficiaryAddress = process.env.BENEFICIARY_ADDRESS;
  const signingClient = await SigningArchwayClient.connectWithSigner(
    network.endpoint,
    wallet
  );
  const wasmCode = fs.readFileSync("./artifacts/test_edt.wasm");
  const encoded = Buffer.from(wasmCode.toString(), "binary").toString("base64");
  const contractData = toByteArray(encoded);

  const uploadResult = await signingClient.upload(
    accountAddress,
    contractData,
    // 5000000000
    // {
    //   amount: [
    //     {
    //       denom: "aconst",
    //       amount: "7000000000000000000",
    //     },
    //   ],
    //   gas: "50000000",
    // }
    "auto"
  );
  if (uploadResult.codeId !== undefined && uploadResult.codeId !== 0) {
    console.log("Storage failed:", uploadResult.logs);
  } else {
    console.log("Storage successful:", uploadResult.transactionHash);
  }
  const codeId = uploadResult.codeId;
  const msg = {
    name: "Enigma Duel Token",
    symbol: "EDT",
    decimals: 9,
    initial_balances: [
      {
        address: accountAddress,
        amount: "10_000_000_000",
      },
    ],
    mint: [{ minter: accountAddress, cap: "100_000_000_000" }],
    marketing: "0",
  };

  const instantiateOptions = {
    memo: "Instantiating the EDT token",
    funds: [{ denom: "aconst", amount: "1000000000000000000" }],
  };
  const instantiateResult = await signingClient.instantiate(
    accountAddress,
    codeId,
    msg,
    "EDT_init",
    "auto",
    instantiateOptions
  );

  console.log("Instantiation successful:", instantiateResult.transactionHash);
}

main();

// the test edt address archway136h687j5gsv40kr57h5xv0p6yvzn032qt7dpkhwg2u40r2s29ycq4heglr
