import task from "@terra-money/terrariums";

task(async ({ deployer, signer, refs }) => {
  //account
  deployer.buildContract("warp-account");
  deployer.optimizeContract("warp-account");

  const id = await deployer.storeCode("warp-account");
  await new Promise((resolve) => setTimeout(resolve, 10000));

  await deployer.storeCode("warp-resolver");
  await new Promise((resolve) => setTimeout(resolve, 10000));

  await deployer.storeCode("warp-templates");
  await new Promise((resolve) => setTimeout(resolve, 10000));

  await deployer.storeCode("warp-controller");
  await new Promise((resolve) => setTimeout(resolve, 10000));

  const instantiateTemplatesMsg = {
    owner: signer.key.accAddress,
    fee_collector: signer.key.accAddress,
    templates: [],
    fee_denom: "uluna",
  }

  await deployer.instantiate("warp-templates", instantiateTemplatesMsg, {
    admin: signer.key.accAddress,
  });
  await new Promise((resolve) => setTimeout(resolve, 10000));

  let resolver_address = await deployer.instantiate("warp-resolver", {}, {
    admin: signer.key.accAddress,
  });
  await new Promise((resolve) => setTimeout(resolve, 10000));

  const instantiateControllerMsg = {
    warp_account_code_id: id,
    fee_denom: "uluna",
    creation_fee: "5",
    cancellation_fee: "5",
    minimum_reward: "10000",
    resolver_address: resolver_address.address,
    t_max: "86400",
    t_min: "86400",
    a_max: "10000",
    a_min: "10000",
    q_max: "10",
  };

  await deployer.instantiate("warp-controller", instantiateControllerMsg, {
    admin: signer.key.accAddress,
  });
  await new Promise((resolve) => setTimeout(resolve, 10000));

  refs.saveRefs();
});
