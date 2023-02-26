import { useEffect, useRef } from "preact/hooks";
import "./app.css";
import { init_canvas } from "./graphics";

export const Proj1 = () => {
  const canvas_ref = useRef<HTMLCanvasElement>();

  useEffect(() => {
    const canvas = canvas_ref.current!;

    const { cleanup } = init_canvas(canvas);
    return () => cleanup();
  });

  return <canvas ref={canvas_ref as any}></canvas>;
};
