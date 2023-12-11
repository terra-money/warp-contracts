import { MsgMigrateContract } from "@terra-money/terra.js";
import task, { info } from "@terra-money/terrariums";

task(async ({ deployer, signer, refs, network }) => {
  deployer.buildContract("warp-controller");
  deployer.optimizeContract("warp-controller");

  await deployer.storeCode("warp-controller");
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const contract = refs.getContract(network, "warp-controller");

  let msg = new MsgMigrateContract(
    signer.key.accAddress,
    contract.address!,
    parseInt(contract.codeId!),
    {
    }
  );

  try {
    let tx = await signer.createAndSignTx({
      msgs: [msg],
    });
    await signer.lcd.tx.broadcast(tx);
  } catch (e) {
    info(JSON.stringify(e));
  }
  info(`Migrated warp-controller contract.`);
});
