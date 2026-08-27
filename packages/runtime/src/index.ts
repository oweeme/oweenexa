export type { ActivationEntry, ActivationManifest, Strategy } from "./manifest";
export { initActivation } from "./activate";
export type { InitActivationOptions, ModuleLoader } from "./activate";
export { activateManually, idle, interaction, load, manual, visible } from "./strategies";
export type { StrategyRunner, Trigger } from "./strategies";
