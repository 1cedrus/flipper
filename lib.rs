#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod flipper {
    use crate::ensure;

    #[derive(Debug, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum FlipperError {
        ZeroSum
    }

    #[ink(event)]
    /// Emitted when the flip function is called.
    pub struct Flipped {
        /// The previous state of the flip.
        old: bool,
        /// The new state of the flip.
        new: bool,
    }

    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        /// Creates a new flipper smart contract with the value being calculate using provided seed.
        #[ink(constructor)]
        pub fn from_seed(seed: Hash) -> Result<Self, FlipperError>{
            let value = seed_to_value(seed)?;

            Ok(Self { value })
        }

        /// Flips the current value, value based on seed.
        #[ink(message)]
        pub fn flip_with_seed(&mut self, seed: Hash) -> Result<bool, FlipperError> {
            let new_value = seed_to_value(seed)?;

            self.value = new_value;
            self.env().emit_event(Flipped { old: !self.value, new: new_value });

            Ok(new_value)
        }

        /// Flips the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;

            self.env().emit_event(Flipped { old: !self.value, new: self.value })
        }

        /// Returns the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }

    }

    fn seed_to_value(seed: Hash) -> Result<bool, FlipperError> {
        let seed: &[u8] = seed.as_ref();
        let sum: u32 = seed.iter().map(|&b| b as u32).sum();

        ensure!(sum != 0, FlipperError::ZeroSum);

        Ok(sum % 2 == 0)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let flipper = Flipper::new_default();
            assert!(!flipper.get());
        }

        #[ink::test]
        fn it_works() {
            let mut flipper = Flipper::new(false);
            assert!(!flipper.get());
            flipper.flip();
            assert!(flipper.get());
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn it_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            let mut constructor = FlipperRef::new(false);
            let contract = client
                .instantiate("flipper", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Flipper>();

            let get = call_builder.get();
            let get_res = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_res.return_value(), false));

            // when
            let flip = call_builder.flip();
            let _flip_res = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("flip failed");

            // then
            let get = call_builder.get();
            let get_res = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_res.return_value(), true));

            Ok(())
        }

        #[ink_e2e::test]
        async fn default_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            let mut constructor = FlipperRef::new_default();

            // when
            let contract = client
                .instantiate("flipper", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Flipper>();

            // then
            let get = call_builder.get();
            let get_res = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_res.return_value(), false));

            Ok(())
        }

        /// This test illustrates how to test an existing on-chain contract.
        ///
        /// You can utilize this to e.g. create a snapshot of a production chain
        /// and run the E2E tests against a deployed contract there.
        /// This process is explained [here](https://use.ink/5.x/basics/contract-testing/chain-snapshot).
        ///
        /// Before executing the test:
        ///   * Make sure you have a node running in the background,
        ///   * Supply the environment variable `CONTRACT_HEX` that points to a deployed
        ///     flipper contract. You can take the SS58 address which `cargo contract
        ///     instantiate` gives you and convert it to hex using `subkey inspect
        ///     <SS58>`.
        ///
        /// The test is then run like this:
        ///
        /// ```
        /// # The env variable needs to be set, otherwise `ink_e2e` will spawn a new
        /// # node process for each test.
        /// $ export CONTRACTS_NODE_URL=ws://127.0.0.1:9944
        ///
        /// $ export CONTRACT_HEX=0x2c75f0aa09dbfbfd49e6286a0f2edd3b4913f04a58b13391c79e96782f5713e3
        /// $ cargo test --features e2e-tests e2e_test_deployed_contract -- --ignored
        /// ```
        ///
        /// # Developer Note
        ///
        /// The test is marked as ignored, as it has the above pre-conditions to succeed.
        #[ink_e2e::test]
        #[ignore]
        async fn e2e_test_deployed_contract<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let addr = std::env::var("CONTRACT_ADDR_HEX")
                .unwrap()
                .replace("0x", "");
            let acc_id = hex::decode(addr).unwrap();
            let acc_id = AccountId::try_from(&acc_id[..]).unwrap();

            // when
            // Invoke `Flipper::get()` from Bob's account
            let call_builder = ink_e2e::create_call_builder::<Flipper>(acc_id);
            let get = call_builder.get();
            let get_res = client.call(&ink_e2e::bob(), &get).dry_run().await?;

            // then
            assert!(matches!(get_res.return_value(), true));
            Ok(())
        }
    }
}

/// Ref: https://github.com/InkSmartContract/BlockchainFoodOrder/blob/main/contracts/foodorder/logic/helpers/helpers.rs
#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr $(,)? ) => {{
        if !$x {
            return Err($y.into());
        }
    }};
}

