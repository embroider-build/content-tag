import type { Preprocessor } from './content_tag';

export * from "./content_tag";

export function createPreprocessor(): Promise<Preprocessor>;
