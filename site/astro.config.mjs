import { defineConfig } from 'astro/config';

function rehypeTableWrap() {
  return (tree) => {
    const visit = (node) => {
      if (!Array.isArray(node.children)) return;
      for (let i = 0; i < node.children.length; i++) {
        const child = node.children[i];
        const parentClass = node.properties?.className;
        const parentClasses = Array.isArray(parentClass)
          ? parentClass
          : parentClass
            ? [parentClass]
            : [];
        if (
          child?.type === 'element' &&
          child.tagName === 'table' &&
          !parentClasses.includes('table-wrap')
        ) {
          node.children[i] = {
            type: 'element',
            tagName: 'div',
            properties: { className: ['table-wrap'] },
            children: [child],
          };
        } else {
          visit(child);
        }
      }
    };
    visit(tree);
  };
}

export default defineConfig({
  site: 'https://pjmagee.github.io',
  base: '/wwm',
  trailingSlash: 'always',
  markdown: {
    rehypePlugins: [rehypeTableWrap],
    shikiConfig: {
      theme: 'github-dark',
    },
  },
});
