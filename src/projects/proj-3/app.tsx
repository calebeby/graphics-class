import { useEffect, useErrorBoundary, useRef, useState } from "preact/hooks";
import "./app.css";
import { init_canvas } from "./graphics";
import obj from "./monkey.obj?raw";
import { load_obj } from "./load-obj";
import * as rust from "./pkg";
import type { GameState } from "./pkg";

interface Props {}

export const TransformDemo = ({}: Props) => {
  // eslint-disable-next-line @typescript-eslint/naming-convention
  const [error, _reset_error] = useErrorBoundary();
  const canvas_ref = useRef<HTMLCanvasElement>(null);
  const [game_state, set_game_state] = useState<GameState | undefined>(
    undefined,
  );

  useEffect(() => {
    rust.default().then(() => {
      const { GameState } = rust;
      set_game_state(new GameState());
    });
  }, [rust.default]);

  useEffect(() => {
    const canvas = canvas_ref.current!;

    if (!game_state) return;
    const { cleanup } = init_canvas(canvas, game_state, load_obj(obj));
    return () => cleanup();
  }, [game_state]);

  return (
    <div class="transform-demo">
      {error}
      <canvas ref={canvas_ref}></canvas>
      <input
        type="range"
        step={0.01}
        min={-5}
        max={5}
        onInput={(e) => game_state?.set_z(e.currentTarget.valueAsNumber)}
      />
      <input
        type="range"
        step={0.01}
        min={-5}
        max={5}
        onInput={(e) => game_state?.set_rotation(e.currentTarget.valueAsNumber)}
      />
    </div>
  );
};
