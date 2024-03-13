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
    minimum_reward: "100000", // 0.1 LUNA
    cancellation_fee_rate: "5",
    resolver_address: resolver_address,
    creation_fee_min: "500000", // 0.5 LUNA
    creation_fee_max: "100000000", // 100 LUNA
    burn_fee_min: "250000", // 0.25 LUNA
    maintenance_fee_min: "250000", // 0.25 LUNA
    maintenance_fee_max: "10000000", // 10 LUNA
    duration_days_min: "7",
    duration_days_max: "90",
    duration_days_limit: "180",
    queue_size_left: "5000",
    queue_size_right: "50000",
    burn_fee_rate: "25", // 25% of job reward
  };

  await deployer.instantiate("warp-controller", instantiateControllerMsg, {
    admin: signer.key.accAddress,
  });
  await new Promise((resolve) => setTimeout(resolve, 10000));

  refs.saveRefs();
});
