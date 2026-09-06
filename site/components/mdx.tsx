import defaultMdxComponents from 'fumadocs-ui/mdx';
import { Tab, Tabs } from 'fumadocs-ui/components/tabs';
import { Callout } from 'fumadocs-ui/components/callout';
import type { MDXComponents } from 'mdx/types';

/**
 * The components every page may use without importing them.
 *
 * The reference pages are generated (`cargo xtask gen ffi`), and a
 * generated page that carried its own imports would repeat them thirty-six
 * times and break the moment a component moved. They are registered here
 * instead, so a generated page is Markdown with tabs in it and nothing
 * else.
 */
export function getMDXComponents(components?: MDXComponents) {
  return {
    ...defaultMdxComponents,
    Tab,
    Tabs,
    Callout,
    ...components,
  } satisfies MDXComponents;
}

export const useMDXComponents = getMDXComponents;

declare global {
  type MDXProvidedComponents = ReturnType<typeof getMDXComponents>;
}
