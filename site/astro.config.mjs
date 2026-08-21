import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'astro/config';

const root = path.dirname(fileURLToPath(import.meta.url));
const installPs1 = path.resolve(root, '../windows/install.ps1');
fs.copyFileSync(installPs1, path.resolve(root, 'public/install.ps1'));
// GitHub Pages (and some dev servers) treat .ps1 as bytes. .txt is text/plain for irm | iex.
fs.copyFileSync(installPs1, path.resolve(root, 'public/install.txt'));

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
  vite: {
    plugins: [
      {
        name: 'install-script-mime',
        configureServer(server) {
          server.middlewares.use((req, res, next) => {
            const url = (req.url || '').split('?')[0];
            if (url.endsWith('/install.ps1') || url.endsWith('/install.txt')) {
              res.setHeader('Content-Type', 'text/plain; charset=utf-8');
            }
            next();
          });
        },
      },
    ],
  },
});
