import { render } from "preact";
import * as p from "../src/projects/proj-3/app";
import "../src/content-page.css";

const root = document.querySelector("#app")!;
render(<p.Proj3 />, root);
