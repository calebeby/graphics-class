import { render } from "preact";
import * as p from "../src/projects/midterm/app";
import "../src/content-page.css";

const root = document.querySelector("#app")!;
render(<p.Midterm />, root);
