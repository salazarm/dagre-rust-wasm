use dagre_rust::{layout, GraphConfig, GraphEdge, GraphNode};
use graphlib_rust::{Graph, GraphOption};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::{collections::HashMap, hash::Hash};
use wasm_bindgen::prelude::*;

pub type GraphId = String;

#[derive(Debug, Clone, Default)]
pub struct IBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Serialize for IBounds {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("IBounds", 4)?;

        // Serialize the fields of IBounds
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;

        state.end()
    }
}

#[derive(Debug, Clone, Default)]
pub struct IPoint {
    pub x: f32,
    pub y: f32,
}
impl Serialize for IPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("IPoint", 2)?;

        // Serialize the fields of IPoint
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;

        state.end()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AssetLayout {
    pub id: GraphId,
    pub bounds: IBounds,
}

impl Serialize for AssetLayout {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AssetLayout", 2)?;

        // Serialize the fields of AssetLayout
        state.serialize_field("id", &self.id)?;
        state.serialize_field("bounds", &self.bounds)?;

        state.end()
    }
}

#[derive(Debug, Clone, Default)]
pub struct GroupLayout {
    pub id: GraphId,
    pub groupName: String,
    pub repositoryName: String,
    pub repositoryLocationName: String,
    pub repositoryDisambiguationRequired: bool,
    pub bounds: IBounds,
}

impl Serialize for GroupLayout {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("GroupLayout", 6)?;

        // Serialize the fields of GroupLayout
        state.serialize_field("id", &self.id)?;
        state.serialize_field("groupName", &self.groupName)?;
        state.serialize_field("repositoryName", &self.repositoryName)?;
        state.serialize_field("repositoryLocationName", &self.repositoryLocationName)?;
        state.serialize_field(
            "repositoryDisambiguationRequired",
            &self.repositoryDisambiguationRequired,
        )?;
        state.serialize_field("bounds", &self.bounds)?;

        state.end()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AssetLayoutEdge {
    pub from: IPoint,
    pub fromId: GraphId,
    pub to: IPoint,
    pub toId: GraphId,
}
impl Serialize for AssetLayoutEdge {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AssetLayoutEdge", 4)?;

        // Serialize the fields of AssetLayoutEdge
        state.serialize_field("from", &self.from)?;
        state.serialize_field("fromId", &self.fromId)?;
        state.serialize_field("to", &self.to)?;
        state.serialize_field("toId", &self.toId)?;

        state.end()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AssetGraphLayout {
    pub width: i32,
    pub height: i32,
    pub edges: Vec<AssetLayoutEdge>,
    pub nodes: HashMap<GraphId, AssetLayout>,
    pub groups: HashMap<String, GroupLayout>,
}

// Implement the Serialize trait for AssetGraphLayout
impl Serialize for AssetGraphLayout {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AssetGraphLayout", 5)?;

        // Serialize the fields of AssetGraphLayout
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("edges", &self.edges)?;
        state.serialize_field("nodes", &self.nodes)?;
        state.serialize_field("groups", &self.groups)?;

        state.end()
    }
}

#[derive(Debug, Clone, Default)]
pub struct GraphData {
    pub nodes: HashMap<GraphId, AssetGraphNode>,
    pub downstream: HashMap<GraphId, HashMap<GraphId, bool>>,
    pub upstream: HashMap<GraphId, HashMap<GraphId, bool>>,
}

#[derive(Debug, Clone, Default)]
pub struct AssetKey {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AssetNode {
    pub __typename: String,
    pub id: String,
    pub groupName: Option<String>,
    pub isExecutable: bool,
    pub hasMaterializePermission: bool,
    pub graphName: Option<String>,
    pub jobNames: Vec<String>,
    pub opNames: Vec<String>,
    pub opVersion: Option<String>,
    pub description: Option<String>,
    pub computeKind: Option<String>,
    pub isPartitioned: bool,
    pub isObservable: bool,
    pub isSource: bool,
    pub repository: Repository,
    pub dependencyKeys: Vec<AssetKey>,
    pub dependedByKeys: Vec<AssetKey>,
    pub assetKey: AssetKey,
}

#[derive(Debug, Clone, Default)]
pub struct Repository {
    pub __typename: String,
    pub id: String,
    pub name: String,
    pub location: RepositoryLocation,
}

#[derive(Debug, Clone, Default)]
pub struct RepositoryLocation {
    pub __typename: String,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct AssetGraphNode {
    pub id: GraphId,
    pub assetKey: AssetKey,
    pub definition: AssetNode,
}

const GROUP_NODE_PREFIX: &str = "group__";
const MARGIN: i32 = 100;

pub struct LayoutAssetGraphOptions {
    pub horizontalDAGs: bool,
}

// #[wasm_bindgen]
pub fn layout_asset_graph(
    graph_data: GraphData,
    opts: LayoutAssetGraphOptions,
) -> AssetGraphLayout {
    let mut g: Graph<GraphConfig, GraphNode, GraphEdge> =
        graphlib_rust::graph::Graph::new(Some(GraphOption {
            compound: Some(true),
            directed: Some(false),
            multigraph: Some(false),
        }));

    let mut nodes: HashMap<GraphId, AssetLayout> = HashMap::new();
    let mut groups: HashMap<String, GroupLayout> = HashMap::new();
    let mut links_to_assets_outside_graphed_set: HashMap<GraphId, bool> = HashMap::new();

    let parent_node_id_for_node = |node: &AssetGraphNode| {
        format!(
            "{}__{}__{}__{}",
            GROUP_NODE_PREFIX,
            node.definition.repository.location.name,
            node.definition.repository.name,
            node.definition.groupName.unwrap_or_default()
        )
    };

    let should_render = |node: Option<&AssetGraphNode>| -> bool {
        if let Some(node) = node {
            node.definition.opNames.len() > 0
        } else {
            false
        }
    };

    let rendered_nodes: Vec<&AssetGraphNode> = graph_data
        .nodes
        .values()
        .filter(|node| should_render(Some(node)))
        .collect();

    for node in &rendered_nodes {
        if let Some(group_name) = &node.definition.groupName {
            let id = parent_node_id_for_node(node);
            groups.insert(
                id.clone(),
                GroupLayout {
                    id: id.clone(),
                    groupName: group_name.clone(),
                    repositoryName: node.definition.repository.name.clone(),
                    repositoryLocationName: node.definition.repository.location.name.clone(),
                    repositoryDisambiguationRequired: false,
                    bounds: IBounds {
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 0.0,
                    },
                },
            );
        }
    }

    let show_groups = groups.len() > 1;
    if show_groups {
        for group_id in groups.keys() {
            g.set_node(group_id.clone(), Some(GraphNode::default()));
        }
    }

    for node in &rendered_nodes {
        let asset_node_dimensions = get_asset_node_dimensions(&node.definition);
        let mut g_node = GraphNode::default();
        g_node.width = asset_node_dimensions.width;
        g_node.height = asset_node_dimensions.height;
        g.set_node(node.id, Some(g_node));

        if show_groups && node.definition.groupName.is_some() {
            g.set_parent(&node.id, Some(parent_node_id_for_node(node)));
        }
    }

    for (upstream_id, graph_data_downstream) in &graph_data.downstream {
        for downstream_id in graph_data_downstream.keys() {
            if !should_render(graph_data.nodes.get(downstream_id))
                && !should_render(graph_data.nodes.get(upstream_id))
            {
                continue;
            }

            g.set_edge(upstream_id, downstream_id, None, None);

            if !should_render(graph_data.nodes.get(downstream_id)) {
                links_to_assets_outside_graphed_set.insert(downstream_id.clone(), true);
            } else if !should_render(graph_data.nodes.get(upstream_id)) {
                links_to_assets_outside_graphed_set.insert(upstream_id.clone(), true);
            }
        }
    }

    for id in links_to_assets_outside_graphed_set.keys() {
        let path: Vec<String> = serde_json::from_str(id).unwrap();
        let label = path.last().unwrap_or(&"".to_string()).clone();
        let asset_link_dimensions = get_asset_link_dimensions(&label, &opts);
        nodes.insert(
            id.clone(),
            AssetLayout {
                id: id.clone(),
                bounds: asset_link_dimensions,
            },
        );
    }

    layout::layout(&mut g);

    let mut max_width = 0;
    let mut max_height = 0;

    for id in g.nodes() {
        let dagre_node = g.node(&id);

        if let Some(dagre_node) = g.node(&id) {
            let bounds = IBounds {
                x: dagre_node.x - dagre_node.width / 2.0,
                y: dagre_node.y - dagre_node.height / 2.0,
                width: dagre_node.width,
                height: dagre_node.height,
            };
            if !id.starts_with(GROUP_NODE_PREFIX) {
                nodes.get_mut(&id).unwrap().bounds = bounds.clone();
            }
            max_width = max_width.max((dagre_node.x + dagre_node.width / 2.0).round() as i32);
            max_height = max_height.max((dagre_node.y + dagre_node.height / 2.0).round() as i32);
        }
    }

    if show_groups {
        for node in &rendered_nodes {
            if let Some(group_name) = &node.definition.groupName {
                let group_id = parent_node_id_for_node(node);
                let group_for_id = groups.get_mut(&group_id).unwrap();
                if group_for_id.bounds.width == 0.0 {
                    group_for_id.bounds = nodes.get(&node.id).unwrap().bounds.clone();
                } else {
                    group_for_id.bounds =
                        extend_bounds(&group_for_id.bounds, &nodes.get(&node.id).unwrap().bounds);
                }
            }
        }
        for group in groups.values_mut() {
            group.bounds = pad_bounds(&group.bounds, &IPoint { x: 15.0, y: 70.0 });
        }
    }

    let mut edges: Vec<AssetLayoutEdge> = Vec::new();

    for edge in g.edges() {
        let v = edge.v;
        let w = edge.w;
        let v_node = g.node(&edge.v).unwrap_or(&GraphNode::default());
        let w_node = g.node(&edge.w).unwrap_or(&GraphNode::default());

        let v_x_inset = if links_to_assets_outside_graphed_set.contains_key(&v) {
            16
        } else {
            24
        };
        let w_x_inset = if links_to_assets_outside_graphed_set.contains_key(&w) {
            16
        } else {
            24
        };

        let asset_layout_edge = if opts.horizontalDAGs {
            AssetLayoutEdge {
                from: IPoint {
                    x: v_node.x + v_node.width / 2.0,
                    y: v_node.y,
                },
                fromId: v.clone(),
                to: IPoint {
                    x: w_node.x - w_node.width / 2.0 - 5.0,
                    y: w_node.y,
                },
                toId: w.clone(),
            }
        } else {
            AssetLayoutEdge {
                from: IPoint {
                    x: v_node.x - v_node.width / 2.0 + v_x_inset as f32,
                    y: v_node.y - 30.0 + v_node.height / 2.0,
                },
                fromId: v.clone(),
                to: IPoint {
                    x: w_node.x - w_node.width / 2.0 + w_x_inset as f32,
                    y: w_node.y + 20.0 - w_node.height / 2.0,
                },
                toId: w.clone(),
            }
        };

        edges.push(asset_layout_edge);
    }

    AssetGraphLayout {
        width: max_width + MARGIN,
        height: max_height + MARGIN,
        edges,
        nodes,
        groups,
    }
}

pub const ASSET_LINK_NAME_MAX_LENGTH: usize = 10;

pub fn get_asset_link_dimensions(label: &str, opts: &LayoutAssetGraphOptions) -> IBounds {
    if opts.horizontalDAGs {
        IBounds {
            x: 0.0,
            y: 0.0,
            width: 32.0 + 8.0 * std::cmp::min(ASSET_LINK_NAME_MAX_LENGTH, label.len()) as f32,
            height: 90.0,
        }
    } else {
        IBounds {
            x: 0.0,
            y: 0.0,
            width: 106.0,
            height: 90.0,
        }
    }
}

pub fn pad_bounds(a: &IBounds, padding: &IPoint) -> IBounds {
    IBounds {
        x: a.x - padding.x,
        y: a.y - padding.y,
        width: a.width + padding.x * 2.0,
        height: a.height + padding.y * 2.0,
    }
}

pub fn extend_bounds(a: &IBounds, b: &IBounds) -> IBounds {
    let xmin = a.x.min(b.x);
    let ymin = a.y.min(b.y);
    let xmax = (a.x + a.width).max(b.x + b.width);
    let ymax = (a.y + a.height).max(b.y + b.height);
    IBounds {
        x: xmin,
        y: ymin,
        width: xmax - xmin,
        height: ymax - ymin,
    }
}

pub const ASSET_NODE_NAME_MAX_LENGTH: usize = 28;

pub fn get_asset_node_dimensions(def: &AssetNode) -> IBounds {
    let width: f32 = 265.0;

    if def.isSource && !def.isObservable {
        IBounds {
            x: 0.0,
            y: 0.0,
            width,
            height: 102.0,
        }
    } else {
        let mut height: f32 = 100.0; // top tags area + name + description

        if def.isSource {
            height += 30.0; // last observed
        } else {
            height += 26.0; // status row
            if def.isPartitioned {
                height += 40.0;
            }
        }

        height += 30.0; // tags beneath

        IBounds {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }
}

fn main() {
    let nodes_json = serde_json::from_str("{\"[\\\"alpha\\\"]\":{\"id\":\"[\\\"alpha\\\"]\",\"assetKey\":{\"__typename\":\"AssetKey\",\"path\":[\"alpha\"]},\"definition\":{\"__typename\":\"AssetNode\",\"id\":\"toys.table_metadata_repository.[\\\"alpha\\\"]\",\"groupName\":\"default\",\"isExecutable\":true,\"hasMaterializePermission\":true,\"repository\":{\"__typename\":\"Repository\",\"id\":\"65485f1efe31f7afff2238ada1c71beba46df2e0\",\"name\":\"table_metadata_repository\",\"location\":{\"__typename\":\"RepositoryLocation\",\"id\":\"toys\",\"name\":\"toys\"}},\"dependencyKeys\":[],\"dependedByKeys\":[],\"graphName\":null,\"jobNames\":[\"__ASSET_JOB\"],\"opNames\":[\"alpha\"],\"opVersion\":null,\"description\":null,\"computeKind\":null,\"isPartitioned\":false,\"isObservable\":false,\"isSource\":false,\"assetKey\":{\"__typename\":\"AssetKey\",\"path\":[\"alpha\"]}}}}");
    let downstream_json = serde_json::from_str("{}");
    let upstream_json = serde_json::from_str("{}");

    let mut nodes: HashMap<String, AssetGraphNode> = HashMap::default();
    let mut downstream: HashMap<String, HashMap<String, bool>> = HashMap::default();
    let mut upstream: HashMap<String, HashMap<String, bool>> = HashMap::default();

    // Handle the Result
    match nodes_json {
        Ok(nodes) => {}
        Err(err) => {
            // Handle the error, for example, print the error message.
            eprintln!("Error: {}", err);
        }
    }
    match downstream_json {
        Ok(downstream) => {}
        Err(err) => {
            // Handle the error, for example, print the error message.
            eprintln!("Error: {}", err);
        }
    }
    match upstream_json {
        Ok(upstream) => {}
        Err(err) => {
            // Handle the error, for example, print the error message.
            eprintln!("Error: {}", err);
        }
    }

    let layout_result = layout_asset_graph(
        GraphData {
            nodes,
            downstream,
            upstream,
        },
        LayoutAssetGraphOptions {
            horizontalDAGs: true,
        },
    );
    println!("{:?}", serde_json::to_string_pretty(&layout_result))
}
