use std::collections::HashMap;
use std::str::FromStr;

use crate::errors::MetaPathDefinitionError;

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Types
//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct MetaPath<T> {
    pub(super) start: PathComponent<T>,
    pub(super) steps: Vec<(PathComponent<T>, PathComponent<T>)>,
}

#[derive(Debug, Clone)]
pub(super) enum PathComponent<T> {
    Typed(T),
    Wildcard
}

impl<T: Copy> Copy for PathComponent<T> {}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Methods
//////////////////////////////////////////////////////////////////////////////////////////////////

impl MetaPath<String> {
    pub fn new(pattern: impl AsRef<str>) -> Result<Self, MetaPathDefinitionError> {
        Self::from_str(pattern.as_ref())
    }
    
    pub(super) fn resolve_types(self, 
                                node_types: &HashMap<String, usize>, 
                                edge_types: &HashMap<String, usize>) -> Result<MetaPath<usize>, MetaPathDefinitionError> 
    {
        let start = self.start.maybe_resolve(node_types, "node")?;
        let steps = self.steps.into_iter()
            .map(|(e, v)| {
                let e_conv = e.maybe_resolve(edge_types, "edge")?;
                let v_conv = v.maybe_resolve(node_types, "node")?;
                Ok((e_conv, v_conv))
            })
            .collect::<Result<Vec<_>, MetaPathDefinitionError>>()?;
        Ok(MetaPath{ start, steps })
    }
}

impl PathComponent<String> {
    fn maybe_resolve(self, 
                     mapping: &HashMap<String, usize>, 
                     kind: &'static str) -> Result<PathComponent<usize>, MetaPathDefinitionError>
    {
        match self {
            PathComponent::Typed(name) => {
                let uid = mapping.get(&name)
                    .copied()
                    .ok_or_else(|| MetaPathDefinitionError::UnknownType {
                        kind: kind.to_string(), name
                    })?;
                Ok(PathComponent::Typed(uid))
            }
            PathComponent::Wildcard => Ok(PathComponent::Wildcard)
        }
    }
}

impl<'a, T: 'a> PathComponent<T> 
where &'a T: Eq
{
    pub(super) fn matches(&'a self, x: &'a T) -> bool {
        match self {
            PathComponent::Typed(y) => x == y,
            PathComponent::Wildcard => true
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Parsing
//////////////////////////////////////////////////////////////////////////////////////////////////

impl FromStr for MetaPath<String> {
    type Err = MetaPathDefinitionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Syntax: 
        // [Node] -{edge}-> [Node]
        // [Node] -> [Node] (edge wildcard)
        // [Node] -{edge}-> [*] (node wildcard)
        let mut stream = s.chars().filter(|c| !c.is_whitespace());
        let start = parse_node_type(&mut stream)?;
        let mut steps = Vec::new();
        let mut first = true;
        while let Some(edge_type) = parse_edge_type(&mut stream, first)? {
            let node_type = parse_node_type(&mut stream)?;
            steps.push((edge_type, node_type));
            first = false;
        }
        Ok(MetaPath { start, steps })
    }
}

fn parse_node_type<I>(stream: &mut I) -> Result<PathComponent<String>, MetaPathDefinitionError> 
where
    I: Iterator<Item=char>
{
    let _ = expect_exact(stream, '[', "node type", true)?;
    parse_node_type_name(stream)
}

fn parse_node_type_name<I>(stream: &mut I) -> Result<PathComponent<String>, MetaPathDefinitionError>
where
    I: Iterator<Item=char>
{
    match stream.next() {
        Some('*') => {
            let _ = expect_exact(stream, ']', "node type", true)?;
            Ok(PathComponent::Wildcard)
        }
        Some(c) if c.is_ascii_alphabetic() => {
            let name = expect_ascii_or_close(
                stream, Some(c), ']', "node type"
            )?;
            Ok(PathComponent::Typed(name))
        }
        Some(_c) => {
            Err(MetaPathDefinitionError::InvalidSyntax {
                detail: "Node type name must be ASCII".to_string()
            })
        }
        None => {
            Err(MetaPathDefinitionError::InvalidSyntax {
                detail: "Unexpected end of input (expected node type name)".to_string()
            })
        }
    }
}

fn parse_edge_type<I>(stream: &mut I, required: bool) -> Result<Option<PathComponent<String>>, MetaPathDefinitionError>
where 
    I: Iterator<Item=char>
{
    if !expect_exact(stream, '-', "edge type", required)? {
        return Ok(None);
    }
    match stream.next() {
        Some('>') => Ok(Some(PathComponent::Wildcard)),
        Some('{') => {
            let name = expect_ascii_or_close(stream, None, '}', "edge type")?;
            let _ = expect_exact(stream, '-', "edge type", true)?;
            let _ = expect_exact(stream, '>', "edge type", true)?;
            Ok(Some(PathComponent::Typed(name)))
        }
        Some(c) => Err(MetaPathDefinitionError::InvalidSyntax {
            detail: format!("Invalid character in edge type: '{c}'")
        }),
        None => Err(MetaPathDefinitionError::InvalidSyntax {
            detail: "Unexpected end of input (expected edge type)".to_string()
        })
    }
}

fn expect_ascii_or_close<I>(stream: &mut I, 
                            initial: Option<char>,
                            closer: char,
                            hint: &'static str) -> Result<String, MetaPathDefinitionError>
where 
    I: Iterator<Item=char>
{
    let mut buffer = match initial {
        Some(c) => vec![c],
        None => Vec::new()
    };
    
    loop {
        match stream.next() {
            Some(c) if c.is_ascii_alphabetic() => {
                buffer.push(c)
            }
            Some(c) if c == closer => {
                break;
            }
            Some(c) => {
                return Err(MetaPathDefinitionError::InvalidSyntax {
                    detail: format!("Expected ASCII character in {hint}, got '{c}'")
                })
            }
            None => {
                return Err(MetaPathDefinitionError::InvalidSyntax {
                    detail: format!("Unexpected end of input in {hint}")
                })
            }
        }
    }
    
    if buffer.is_empty() {
        return Err(MetaPathDefinitionError::InvalidSyntax {
            detail: format!("Empty type name in {hint}")
        })
    }
    Ok(buffer.into_iter().collect())
}

fn expect_exact<I>(stream: &mut I,
                   e: char,
                   hint: &'static str, 
                   required: bool) -> Result<bool, MetaPathDefinitionError>
where
    I: Iterator<Item=char>
{
    match stream.next() {
        Some(c) if c == e => Ok(true),
        Some(c) => {
            Err(MetaPathDefinitionError::InvalidSyntax {
                detail: format!("Invalid syntax in {hint}: expected '{e}', got '{c}'")
            })
        }
        None => {
            if required {
                Err(MetaPathDefinitionError::InvalidSyntax {
                    detail: format!("Unexpected end of input while parsing {hint} (expected '{e}')")
                })
            } else {
                Ok(false)
            }
        }
    }
}