import type { Preprocessor } from './content_tag';

export * from "./content_tag";

export async function createPreprocessor(): Promise<Preprocessor>;
