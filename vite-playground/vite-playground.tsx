import { render } from "preact";
import * as p from "../src/pages/proj-2/app";
import "../src/content-page.css";

const root = document.querySelector("#app")!;
render(<p.TransformDemo initial_transforms={[]} />, root);
