// One page per tool or script, made from its own help (its docstring or header comment, as
// ./gameframe reads it) when the site builds.
import { toolPage, tools } from '../.vitepress/catalog.mts'

export default {
  watch: ['../../tools/**/*.py', '../../tools/**/*.java', '../../scripts/*.sh'],
  paths() {
    return tools().map(t => ({ params: { tool: t.name, source: t.path }, content: toolPage(t) }))
  },
}
