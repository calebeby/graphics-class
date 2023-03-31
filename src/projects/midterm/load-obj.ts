export type Vertex = [x: number, y: number, z: number];
export type Face = Vertex[];

export const load_obj = (obj: string) => {
  const lines = obj.split("\n");
  const vertices: Vertex[] = [];
  const faces: Face[] = [];
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const line_num = i + 1;
    if (line.startsWith("#") || line.trim() === "") continue;
    const chunks = line.split(/\s+/g);
    const command = chunks[0];
    if (
      command === "mtllib" ||
      command === "o" ||
      command === "vn" ||
      command === "vt" ||
      command === "s"
    ) {
      // ignore/no need to handle (at least for now)
    } else if (command === "v") {
      // @ts-expect-error
      vertices.push([...chunks.slice(1, 4).map((chunk) => Number(chunk)), 1]);
    } else if (command === "f") {
      const face_vertices = chunks.slice(1);
      if (face_vertices.length !== 3)
        throw new Error(
          "Only triangles are supported in loading obj files for now",
        );
      faces.push(
        face_vertices.map((chunk) => {
          // eslint-disable-next-line @typescript-eslint/naming-convention
          const [vertex_index, _texture_index, _normal_index] = chunk
            .split("/")
            .map((c) => Number(c));
          return vertices[vertex_index - 1];
        }),
      );
    } else {
      throw new Error(`Unrecognized command on line ${line_num}: ${command}`);
    }
  }
  return faces;
};
