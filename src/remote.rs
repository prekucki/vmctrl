use super::command::Ssh;
use super::uri::DriverFactory;
use super::{ssh, Driver, FromCommandRunner, Machine};

pub struct RemoteFactory<D: FromCommandRunner>(D);

fn parse_ssh(path: &str) -> Option<(&str, &str)> {
    if path.starts_with("//") {
        return parse_ssh(&path[2..]);
    }

    if let Some(p) = path.find(":") {
        return Some((&path[0..p], &path[p + 1..]));
    }

    if let Some(p) = path.find("/") {
        return Some((&path[0..p], &path[p + 1..]));
    }

    None
}

impl<R: FromCommandRunner<Command = Ssh, Output = D>, D: Driver> DriverFactory for RemoteFactory<R>
where
    D::Machine: 'static,
{
    fn machine_for_uri(&self, uri: &str) -> Option<Box<Machine>> {
        if let Some((host, path)) = parse_ssh(uri) {
            let driver = self.0.from_cmd(ssh(host));

            return match driver.from_path(path) {
                Ok(m) => Some(Box::new(m)),
                Err(_) => None,
            };
        }
        None
    }
}

impl<R: FromCommandRunner<Command = Ssh, Output = D>, D: Driver> From<R> for RemoteFactory<R>
where
    D::Machine: 'static,
{
    fn from(factory: R) -> Self {
        RemoteFactory(factory)
    }
}

impl<R: FromCommandRunner<Command = Ssh, Output = D> + 'static, D: Driver> From<R>
    for Box<DriverFactory>
where
    D::Machine: 'static,
{
    fn from(factory: R) -> Self {
        let f: RemoteFactory<_> = factory.into();
        Box::new(f)
    }
}
