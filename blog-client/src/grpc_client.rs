use blog_proto::blog_service_client::BlogServiceClient;
use tonic::transport::Channel;

pub type GrpcClient = BlogServiceClient<Channel>;
