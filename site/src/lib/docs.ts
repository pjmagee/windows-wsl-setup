import { getCollection, type CollectionEntry } from 'astro:content';

export type Doc = CollectionEntry<'docs'>;

export async function allDocs(): Promise<Doc[]> {
  const docs = await getCollection('docs');
  return docs.sort((a, b) => a.data.order - b.data.order);
}

export function groups(docs: Doc[]) {
  const map = new Map<string, Doc[]>();
  for (const d of docs) {
    const g = d.data.group;
    if (!map.has(g)) map.set(g, []);
    map.get(g)!.push(d);
  }
  return [...map.entries()];
}
