// Copyright 2019-2020 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.


use sp_core::{crypto::SecretStringError, Pair, ecdsa, sr25519, ed25519};
use sp_runtime::{MultiSignature, MultiSigner};
use structopt::StructOpt;
use std::str::FromStr;
use failure::Fail;

#[derive(Debug, StructOpt)]
#[structopt(
name = "multisig",
author = "Moonbeam Team <admin@parity.io>",
about = "Utility for generating Multisignatures",
)]
pub enum MultiSig {
	#[structopt(name = "generate-signature")]
	GenerateSignature(GenerateSignature),
	#[structopt(name = "generate-signer")]
	GenerateSigner(GenerateSigner),
}

/// Command for generating a multisignature.
#[derive(Debug, StructOpt)]
pub struct GenerateSignature {
	#[structopt(short, long)]
	pub private_key: String,

	#[structopt(long)]
	pub data: String,

	#[structopt(long)]
	pub algorithm: String,
}

/// Command for generating a multisigner.
#[derive(Debug, StructOpt)]
pub struct GenerateSigner {
	#[structopt(short, long)]
	pub private_key: String,


	#[structopt(long)]
	pub algorithm: String,
}

#[derive(Debug, Fail)]
pub enum Error {
	#[fail(display = "Wrong algorithm provided")]
	InvalidAlgo,
	#[fail(display = "Invalid secret provided")]
	InvalidSec {e: SecretStringError},
}

#[derive(Debug, StructOpt)]
pub enum SignatureAlgorithm {
	Ecdsa,
	Sr25519,
	Ed25519
}

impl FromStr for SignatureAlgorithm {
	type Err = Error;

	fn from_str(s: &str) -> Result<SignatureAlgorithm, Self::Err> {
		match s {
			"ecdsa" => Ok(SignatureAlgorithm::Ecdsa),
			"sr25519" => Ok(SignatureAlgorithm::Sr25519),
			"ed25519" => Ok(SignatureAlgorithm::Ed25519),
			_ => Err(Error::InvalidAlgo),
		}
	}
}

fn generic_sign<TPair: Pair>(
	private_key: String,
	data: String,
) -> Result<MultiSignature, Error>
	where MultiSignature: From<<TPair as sp_core::Pair>::Signature>
{

	let (pair, _) = TPair::from_phrase(&private_key, None).map_err(|e| Error::InvalidSec{e})?;
	Ok(MultiSignature::from(pair.sign(&hex::decode(data.clone()).unwrap_or(data.as_bytes().to_vec()))))
}

pub fn generate_account(
	private_key: String,
	algorithm: SignatureAlgorithm,
) -> Result<MultiSigner, Error>
{
	match algorithm {
		SignatureAlgorithm::Ecdsa => generic_generate_account::<ecdsa::Pair>(private_key),
		SignatureAlgorithm::Sr25519 => generic_generate_account::<sr25519::Pair>(private_key),
		SignatureAlgorithm::Ed25519 => generic_generate_account::<ed25519::Pair>(private_key),
	}
}

fn generic_generate_account<TPair: Pair>(
	private_key: String,
) -> Result<MultiSigner, Error>
	where MultiSigner: From<<TPair as sp_core::Pair>::Public>
{
	let (pair, _) = TPair::from_phrase(&private_key, None).map_err(|e| Error::InvalidSec{e})?;
	Ok(MultiSigner::from(pair.public()))
}

pub fn sign(
	private_key: String,
	data: String,
	algorithm: SignatureAlgorithm,
) -> Result<MultiSignature, Error>
{
	match algorithm {
		SignatureAlgorithm::Ecdsa => generic_sign::<ecdsa::Pair>(private_key, data),
		SignatureAlgorithm::Sr25519 => generic_sign::<sr25519::Pair>(private_key, data),
		SignatureAlgorithm::Ed25519 => generic_sign::<ed25519::Pair>(private_key, data),
	}
}

/// Run the subkey command, given the apropriate runtime.
pub fn run() -> Result<(), Error> {
	match MultiSig::from_args() {
		MultiSig::GenerateSignature(params) => {
			let signature = sign(params.private_key, params.data, params.algorithm.parse()?);
			println!("{:?}", signature);
			Ok(())
		},
		MultiSig::GenerateSigner(params) => {
			let signer = generate_account(params.private_key,params.algorithm.parse()?);
			println!("{:?}", signer);
			Ok(())
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_runtime::traits::Verify;
	use sp_runtime::traits::IdentifyAccount;

	#[test]
	fn test_generic_sign_ecdsa() {
		let mnemonic =
			"bottom drive obey lake curtain smoke basket hold race lonely fit walk".to_string();
		let data = "this is what I want to sigb";
		let signature = sign(mnemonic.clone(),data.to_string(), SignatureAlgorithm::Ecdsa).unwrap();

		let (pair, _) = ecdsa::Pair::from_phrase(&mnemonic, None).unwrap();
		let multi_signer = MultiSigner::from(pair.public());

		assert!(signature.verify(data.as_bytes(), &multi_signer.into_account()));
	}
	#[test]
	fn test_generic_sign_sr25519() {
		let mnemonic =
			"bottom drive obey lake curtain smoke basket hold race lonely fit walk".to_string();
		let data = "this is what I want to sigb";
		let signature = sign(mnemonic.clone(),data.to_string(), SignatureAlgorithm::Sr25519).unwrap();

		let (pair, _) = sr25519::Pair::from_phrase(&mnemonic, None).unwrap();
		let multi_signer = MultiSigner::from(pair.public());

		assert!(signature.verify(data.as_bytes(), &multi_signer.into_account()));
	}
	#[test]
	fn test_generic_sign_ed25519() {
		let mnemonic =
			"bottom drive obey lake curtain smoke basket hold race lonely fit walk".to_string();
		let data = "this is what I want to sigb";
		let signature = sign(mnemonic.clone(),data.to_string(), SignatureAlgorithm::Ed25519).unwrap();

		let (pair, _) = ed25519::Pair::from_phrase(&mnemonic, None).unwrap();
		let multi_signer = MultiSigner::from(pair.public());

		assert!(signature.verify(data.as_bytes(), &multi_signer.into_account()));
	}
	#[test]
	fn test_same_signature_as_hex() {
		let mnemonic =
			"bottom drive obey lake curtain smoke basket hold race lonely fit walk".to_string();
		let data = "48656c6c6f20776f726c6421";
		let signature = sign(mnemonic.clone(),data.to_string(), SignatureAlgorithm::Sr25519).unwrap();

		let (pair, _) = sr25519::Pair::from_phrase(&mnemonic, None).unwrap();
		let multi_signer = MultiSigner::from(pair.public());
		let data_as_str = "Hello world!";
		assert!(signature.verify(data_as_str.as_bytes(), &multi_signer.into_account()));
	}
}