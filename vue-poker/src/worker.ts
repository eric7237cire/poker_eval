import * as Comlink from "comlink";
//import { detect } from "detect-browser";

type Mod = typeof import("../rsw-hello/pkg/rsw_hello");

const createHandler = (mod: Mod) => {
  return {
    //game: mod.GameManager.new(),

    init(player: string) {
        return mod.hello(" huh");
    },
    sayHello(player: string) : string {
      return mod.hello("from the worker");
    },

    

    
  };
};

// const isMTSupported = () => {
//   const browser = detect();
//   return !(browser && (browser.name === "safari" || browser.os === "iOS"));
// };

let mod: Mod | null = null;
export type Handler = ReturnType<typeof createHandler>;

const initHandler = async (num_threads: number) => {
//   if (isMTSupported()) {
//     mod = await import("../pkg/solver-mt/solver.js");
//     await mod.default();
//     await (mod as ModMT).initThreadPool(num_threads);
//   } else {
//     mod = await import("../pkg/solver-st/solver.js");
//     await mod.default();
//   }

  mod = await import("../rsw-hello/pkg/rsw_hello");
  await mod.default();

  return Comlink.proxy(createHandler(mod));
};

const beforeTerminate = async () => {
//   if (isMTSupported()) {
//     await (mod as ModMT).exitThreadPool();
//   }
};

export interface WorkerApi {
  initHandler: typeof initHandler;
  beforeTerminate: typeof beforeTerminate;
}

Comlink.expose({ initHandler, beforeTerminate });
