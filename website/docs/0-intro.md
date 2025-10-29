---
id: intro
title: Fungible Tokens Zero to Hero
sidebar_label: Introduction
description: "Master NEAR fungible tokens from pre-deployed contracts to building fully-featured FT smart contracts."
---

In this _Zero to Hero_ series, you'll find a set of tutorials covering every aspect of a fungible token (FT) smart contract. You'll start by interacting with a pre-deployed contract and by the end you'll have built a fully-fledged FT smart contract that supports every extension of the standards.

---

## Prerequisites

To complete these tutorials successfully, you'll need:

- [Rust](https://docs.near.org/smart-contracts/quickstart#prerequisites)
- [A NEAR wallet](https://testnet.mynearwallet.com)
- [NEAR-CLI](https://docs.near.org/tools/near-cli#installation)
- [cargo-near](https://github.com/near/cargo-near)

:::info New to Rust?
If you are new to Rust and want to dive into smart contract development, our [Quick-start guide](https://docs.near.org/smart-contracts/quickstart) is a great place to start.
:::

---

## Overview

These are the steps that will bring you from **_Zero_** to **_Hero_** in no time! 💪

| Step | Name                                                         | Description                                                                                                                                     |
| ---- | ------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | [Pre-deployed contract](0-predeployed.md) | Receive FTs without the need to code, create, or deploy a smart contract.                                                                       |
| 2    | [Contract architecture](1-skeleton.md)             | Learn the basic architecture of the FT smart contract and compile the code.                                                                     |
| 3    | [Defining a Token](2-define-a-token.md)          | Flesh out what it means to have a FT and how you can customize your own.                                                                         |
| 4    | [Circulating Supply](3-circulating-supply.md)      | Learn how to create an initial supply and have the token show up in your wallet.                                                                |
| 5    | [Registering Accounts](4.storage.md)  | Explore how you can implement and understand the storage management standard to avoid malicious users from draining your funds.                 |
| 6    | [Transferring FTs](5.transfers.md)                 | Learn how to transfer FTs and discover some of the true powers that the core standard brings                                                    |
| 7    | [Marketplace](6-marketplace.md)                    | Learn about how common marketplaces operate on NEAR and dive into some of the code that allows buying and selling NFTs by using Fungible Tokens. |

<!--
1. [Events](/tutorials/fts/events): in this tutorial you'll explore the events extension, allowing the contract to react on certain events.
1. [Marketplace](/tutorials/fts/marketplace): in the last tutorial you'll be exploring some key aspects of the marketplace contract.
-->

---

## Next steps

Ready to start? Jump to the [Pre-deployed Contract](0-predeployed.md) tutorial and begin your learning journey!

If you already know about fungible tokens and smart contracts, feel free to skip and jump directly to the tutorial of your interest. The tutorials have been designed so you can start at any given point!
