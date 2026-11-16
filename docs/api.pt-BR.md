# Referência da API: visão geral

[English](api.md) · [Referência completa por crate](api/index.md)

Esta página explica como o código do ConstructCraft está organizado e onde ficam seus tipos principais. A [referência completa](api/index.md) lista, crate a crate e módulo a módulo, todo item que não é privado: funções, structs, enums, traits, apelidos de tipo, constantes e reexportações, com suas assinaturas e documentação, os campos das structs, as variantes dos enums, os métodos de cada tipo e as traits que cada tipo implementa. Ela é gerada a partir do código-fonte por `cargo run -p api_doc` (`tools/api_doc`), então diz sempre o que dizem os comentários de documentação; para mudá-la, edite os comentários e rode o gerador de novo. A referência completa está em inglês, como o código.

Se você quer *usar* o ConstructCraft de fora (scripts, linha de comando, agentes de IA), o que precisa é a [referência de comandos](commands.md) e o [guia de automação](automacao.md): os comandos são a interface estável. Esta referência é para quem trabalha no próprio código.

## Classes e herança em Rust

O ConstructCraft é escrito em Rust, que não tem classes nem herança. As mesmas ideias são expressas com outras peças, e a referência se organiza em torno delas:

| Ideia de orientação a objetos | Em Rust | Nesta referência |
|---|---|---|
| Classe com atributos | `struct` | *struct*, com seus **Fields** (campos) |
| Hierarquia de classes com um conjunto fechado de subclasses | `enum` cujas variantes carregam dados | *enum*, com suas **Variants** (variantes) |
| Métodos | funções em blocos `impl Tipo { … }`, que podem estar em qualquer módulo do crate | **Methods** do tipo; *(in `módulo`)* diz onde o método está definido quando não é no módulo do próprio tipo |
| Interface / classe base abstrata | `trait` | *trait*, com seus **Required and provided items** (itens exigidos e fornecidos) e **Implemented by** (implementada por) |
| Herdar uma interface | uma supertrait: `trait Command: DeserializeOwned + JsonSchema` | a assinatura da trait |
| Implementar uma interface | `impl Trait for Tipo` | **Traits** do tipo |
| Comportamento padrão gerado | `#[derive(Clone, Debug, Serialize, …)]` | também listado em **Traits** |
| Membros estáticos | funções associadas (sem `self`) e constantes associadas | em **Methods** |

Dois padrões fazem o papel da herança em todo o código:

- **Enums como hierarquias fechadas.** Um elemento do modelo é uma única struct `Element` (id, uid, nível, tipo, fases, opção de projeto, propriedades) cujo `kind: ElementKind` diz que tipo de elemento ele é: `ElementKind::Wall(Wall)`, `Floor(Floor)`, `Door(Hosted)`, `Pipe(MepSegment)`, `Truss(Truss)` e assim por diante. O código que chamaria um método virtual faz `match` na variante. Tipos (`ElementType` com `TypeData`), vistas (`View` com `ViewKind`) e muitas coisas menores seguem o mesmo padrão.
- **Traits como interfaces abertas.** Onde novas implementações são acrescentadas com frequência, o código usa uma trait. As principais:
  - [`Command`](api/cc_engine.md#cc_engine-registry-command) (`cc_engine`): cada comando do motor é uma struct de parâmetros que a implementa (várias centenas de implementações).
  - [`Extension`](api/cc_engine.md#cc_engine-extension-extension) (`cc_engine`): acrescenta comandos a uma sessão; implementada pelo hospedeiro de plugins.
  - [`MemberHost`](api/cc_core.md#cc_core-member_host-memberhost) (`cc_core`): um elemento que gera barras estruturais (galpões metálicos, treliças, contraventamentos, estruturação de paredes).

## Crates

O código é um workspace do Cargo formado por crates, cada um uma biblioteca (ou um programa) com uma responsabilidade. Cada crate usa apenas os que vêm antes dele nesta ordem, então o `cc_core` não depende de nenhum outro crate e nenhum depende do `cc_app`:

| Crate | Usa (crates do workspace) |
|---|---|
| `cc_core` | — |
| `cc_skp` | — |
| `cc_script` | `cc_core` |
| `cc_cad` | `cc_core` |
| `cc_geom` | `cc_core`, `cc_script` |
| `cc_views`, `cc_ifc`, `cc_render` | `cc_core`, `cc_geom` |
| `cc_raytracing` | `cc_core`, `cc_geom`, `cc_views` |
| `cc_engine` | `cc_core`, `cc_cad`, `cc_geom`, `cc_ifc`, `cc_raytracing`, `cc_script`, `cc_skp`, `cc_views` |
| `cc_plugins` | `cc_engine`, `cc_script`, `cc_skp` |
| `cc_cli` | `cc_core`, `cc_engine`, `cc_plugins` |
| `cc_app` | `cc_core`, `cc_engine`, `cc_geom`, `cc_ifc`, `cc_plugins`, `cc_raytracing`, `cc_render`, `cc_script`, `cc_views` |

O que cada crate contém:

| Crate | O que contém | Tipos principais |
|---|---|---|
| [`cc_core`](api/cc_core.md) | O modelo BIM, fonte única da verdade: projeto, níveis, materiais, tipos, elementos, vistas, desfazer/refazer, arquivos. Não sabe nada de geometria nem de interface. | [`Project`](api/cc_core.md#cc_core-project-project), [`Element`](api/cc_core.md#cc_core-element-element), [`ElementKind`](api/cc_core.md#cc_core-element-elementkind), [`ElementType`](api/cc_core.md#cc_core-types-elementtype), [`TypeData`](api/cc_core.md#cc_core-types-typedata), [`View`](api/cc_core.md#cc_core-view-view), [`ViewKind`](api/cc_core.md#cc_core-view-viewkind), [`Document`](api/cc_core.md#cc_core-document-document), [`EditCtx`](api/cc_core.md#cc_core-document-editctx), [`Transaction`](api/cc_core.md#cc_core-document-transaction), [`ComponentDef`](api/cc_core.md#cc_core-component-componentdef), [`Sketch`](api/cc_core.md#cc_core-sketch-sketch), [`ScheduleDef`](api/cc_core.md#cc_core-schedule-scheduledef) |
| [`cc_geom`](api/cc_geom.md) | A geometria derivada do modelo, reconstruída quando ele muda: junções de paredes, sólidos, telhados, ambientes, estrutura, armaduras, redes de instalações, terreno. | [`ModelGeometry`](api/cc_geom.md#cc_geom-build-modelgeometry), [`WallGeom`](api/cc_geom.md#cc_geom-wall-wallgeom), [`Solid`](api/cc_geom.md#cc_geom-solid-solid) |
| [`cc_views`](api/cc_views.md) | Vistas como consultas ao modelo: plantas, elevações, cortes, tabelas, folhas, anotações; exportação para SVG, PDF, PNG e DXF. | [`generate`](api/cc_views.md#cc_views-generate), [`Drawing2D`](api/cc_views.md#cc_views-drawing-drawing2d), [`Resolver`](api/cc_views.md#cc_views-graphics-resolver) |
| [`cc_render`](api/cc_render.md) | O renderizador em GPU da vista 3D (wgpu). | [`Renderer3D`](api/cc_render.md#cc_render-renderer-renderer3d) |
| [`cc_raytracing`](api/cc_raytracing.md) | Renderização fotorrealista na CPU (path tracing). | [`RenderScene`](api/cc_raytracing.md#cc_raytracing-scene-renderscene) |
| [`cc_ifc`](api/cc_ifc.md) | Exportação e importação IFC, com leitor e escritor STEP próprios. | |
| [`cc_cad`](api/cc_cad.md) | Leitura de desenhos DXF, DWG e SVG para o desenho neutro do modelo. | [`CadDrawing`](api/cc_core.md#cc_core-cad-caddrawing) (em `cc_core`) |
| [`cc_skp`](api/cc_skp.md) | Leitura e escrita de arquivos do SketchUp, sem dependências externas. | [`SkpFile`](api/cc_skp.md#cc_skp-parser-skpfile) |
| [`cc_script`](api/cc_script.md) | O Lua 5.4 isolado usado pelos componentes com script e pelos plugins. | |
| [`cc_engine`](api/cc_engine.md) | O motor: uma sessão e o catálogo de comandos que todos os clientes chamam; grafos de programação visual; tradução da interface. | [`Session`](api/cc_engine.md#cc_engine-session-session), [`Command`](api/cc_engine.md#cc_engine-registry-command), [`EngineError`](api/cc_engine.md#cc_engine-error-engineerror), [`Graph`](api/cc_engine.md#cc_engine-graph-graph), [`Evaluator`](api/cc_engine.md#cc_engine-graph-eval-evaluator) |
| [`cc_plugins`](api/cc_plugins.md) | Scripts Lua e plugins sobre o motor. | [`PluginHost`](api/cc_plugins.md#cc_plugins-plugin-pluginhost) |
| [`cc_cli`](api/cc_cli.md) | O programa `ccad`: comandos, scripts e o servidor MCP, sem interface gráfica. | |
| [`cc_app`](api/cc_app.md) | O aplicativo de desktop (egui). | [`AppState`](api/cc_app.md#cc_app-state-appstate), [`UiCommand`](api/cc_app.md#cc_app-commands-uicommand), [`Tool`](api/cc_app.md#cc_app-tools-tool) |

## Como uma edição percorre o código

1. Um cliente (o aplicativo, o `ccad`, o servidor MCP, um script Lua ou um grafo) pede à `Session` que execute um comando pelo nome com parâmetros JSON, ou uma struct de comando tipada.
2. O `run` do comando (a trait `Command`) resolve suas referências (níveis, tipos e vistas por id ou nome) e edita o modelo por `commands::edit`, que chama `Document::try_edit`.
3. Dentro da edição, um `EditCtx` altera o `Project`; ao terminar, `EditCtx::finish` mantém os dados derivados em dia (cotas, vínculos, estacas, shafts, hospedeiros de barras, fases, opções de projeto, modelos de vista…) e o resultado passa pela verificação de reparo. Um comando que deixaria o modelo inválido é recusado sem criar passo de desfazer; senão a mudança fica registrada como uma única `Transaction` reversível.
4. A revisão do documento muda, então `ModelGeometry::build` (`cc_geom`) reconstrói a geometria e `cc_views::generate` redesenha as vistas exibidas, cada uma como um `Drawing2D`; a vista 3D vai para o `Renderer3D`.

## Convenções que você vai encontrar na referência

- Comprimentos em metros, ângulos em graus, pontos `[x, y]` em planta, salvo quando o comentário diz outra coisa (medidas no papel são em milímetros).
- Erros são valores: as funções devolvem `Result` ou `Option` em vez de entrar em pânico; os erros do motor são `EngineError { code, message }`.
- Itens marcados `pub(crate)` são internos ao seu crate; aparecem porque a referência é para quem trabalha no código.
- O código de testes (`#[cfg(test)]`) fica de fora.
