use select::node::Node;
use select::predicate::Attr;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub size: String,
    pub seeds: usize,
    pub leech: usize,
    pub uploader: String,
}

impl<'a> TryFrom<Node<'a>> for Item {
    type Error = ();

    fn try_from(value: Node) -> Result<Self, Self::Error> {
        let mut children = value.children();

        children.next();

        let title_node = children.next().unwrap();

        let id = title_node
            .find(Attr("href", ()))
            .nth(0)
            .unwrap()
            .attr("href")
            .unwrap()
            .to_string();

        let name = title_node
            .find(Attr("title", ()))
            .last()
            .unwrap()
            .attr("title")
            .unwrap()
            .to_string();

        children.next();

        let size = children.next().unwrap().text();
        let seeds = children.next().unwrap().text().parse::<usize>().unwrap();
        let leech = children.next().unwrap().text().parse::<usize>().unwrap();

        children.next();

        let uploader = children.next().unwrap().text();

        Ok(Self {
            name,
            size,
            id,
            seeds,
            leech,
            uploader,
        })
    }
}
