import { render } from "preact";
import * as p from "../src/projects/final/app";
import "../src/content-page.css";

const root = document.querySelector("#app")!;
render(<p.Final />, root);
