import { render } from "preact";
import * as p from "../src/projects/proj-4/app";
import "../src/content-page.css";

const root = document.querySelector("#app")!;
render(<p.Proj4 />, root);
