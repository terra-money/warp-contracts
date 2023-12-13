import task from "@terra-money/terrariums";

task(async ({ deployer, signer, refs }) => {
  deployer.buildContract("warp-controller");
  deployer.optimizeContract("warp-controller");

  const account_contract_id = await deployer.storeCode("warp-account");
  await new Promise((resolve) => setTimeout(resolve, 10000));

  await deployer.storeCode("warp-resolver");
  await new Promise((resolve) => setTimeout(resolve, 10000));

  await deployer.storeCode("warp-templates");
  await new Promise((resolve) => setTimeout(resolve, 10000));

  await deployer.storeCode("warp-controller");
  await new Promise((resolve) => setTimeout(resolve, 10000));

  const account_tracker_id = await deployer.storeCode("warp-account-tracker");
  await new Promise((resolve) => setTimeout(resolve, 10000));

  const instantiateTemplatesMsg = {
    owner: signer.key.accAddress,
    fee_collector: signer.key.accAddress,
    templates: [],
    fee_denom: "uluna",
  };

  await deployer.instantiate("warp-templates", instantiateTemplatesMsg, {
    admin: signer.key.accAddress,
  });
  await new Promise((resolve) => setTimeout(resolve, 10000));

  let resolver_address = await deployer.instantiate(
    "warp-resolver",
    {},
    {
      admin: signer.key.accAddress,
    }
  );
  await new Promise((resolve) => setTimeout(resolve, 10000));

  const instantiateControllerMsg = {
    fee_denom: "uluna",
    fee_collector: signer.key.accAddress,
    warp_account_code_id: account_contract_id,
    account_tracker_code_id: account_tracker_id,
    minimum_reward: "10000",
    cancellation_fee_rate: "5",
    resolver_address: resolver_address,
    creation_fee_min: "500000",
    creation_fee_max: "100000000",
    burn_fee_min: "100000",
    maintenance_fee_min: "50000",
    maintenance_fee_max: "10000000",
    duration_days_min: "10",
    duration_days_max: "100",
    queue_size_left: "5000",
    queue_size_right: "50000",
    burn_fee_rate: "25",
  };

  await deployer.instantiate("warp-controller", instantiateControllerMsg, {
    admin: signer.key.accAddress,
  });
  await new Promise((resolve) => setTimeout(resolve, 10000));

  refs.saveRefs();
});
