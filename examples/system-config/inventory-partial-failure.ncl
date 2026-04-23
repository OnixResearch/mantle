{
  machines = {
    server1 = {
      system = "x86_64-linux",
      class = "nixos",
    },
    server2 = {
      system = "x86_64-linux",
      class = "nixos",
    },
  },
  services = {
    sshd = {
      instances = [
        {
          machine = "server1",
          role = "default",
          settings = { port = "bad-port" },
          tags = ["base"],
        },
        {
          machine = "server2",
          role = "default",
          settings = { port = 22 },
          tags = ["base"],
        },
      ],
    },
    firewall = {
      instances = [
        {
          machine = "server1",
          role = "default",
          settings = { allowed_ports = [80, 8080] },
          tags = ["network"],
        },
        {
          machine = "server2",
          role = "default",
          settings = { allowed_ports = [80] },
          tags = ["network"],
        },
      ],
    },
    nginx = {
      instances = [
        {
          machine = "server1",
          role = "default",
          settings = { port = 8080 },
          tags = ["web"],
        },
        {
          machine = "server2",
          role = "default",
          settings = { port = 8081 },
          tags = ["web"],
        },
      ],
    },
  },
}
