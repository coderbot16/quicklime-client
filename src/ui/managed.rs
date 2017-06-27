use gfx::{Encoder, Resources, CommandBuffer, Slice, IndexBuffer};
use gfx::memory::{Usage, TRANSFER_DST};
use gfx::handle::Buffer;
use gfx::traits::{Factory, FactoryExt};
use gfx::buffer::Role;
use ui::render::Vertex;

// step: 128 vertices (4096 bytes, 42 triangles + 2 extra vertices)
const ALLOC_STEP: usize = 128;

#[derive(Debug)]
struct Zone {
	start: usize,
	size: usize
}

pub struct ManagedBuffer<R> where R: Resources {
	local: Vec<Vertex>,
	remote: Buffer<R, Vertex>,
	zones: Vec<(Zone, bool)>,
	tail: usize
}

impl<R> ManagedBuffer<R> where R: Resources {
	pub fn new<F>(factory: &mut F) -> Self where F: Factory<R> {
		ManagedBuffer {
			local: Vec::new(),
			remote: factory.create_buffer(ALLOC_STEP, Role::Vertex, Usage::Dynamic, TRANSFER_DST).unwrap(),
			zones: Vec::new(),
			tail: 0
		}
	}
	
	pub fn new_zone(&mut self) -> usize {
		self.zones.push((Zone {start: self.tail, size: 0}, true));
		self.zones.len() - 1
	}
	
	pub fn replace_zone(&mut self, buffer: &[Vertex], zone: usize) {
		let (ref mut zone, ref mut dirty) = self.zones[zone];
		*dirty = true;
		
		if zone.size == buffer.len() {
			let slice = &mut self.local[zone.start..zone.start + zone.size];
			slice.copy_from_slice(buffer);
		} else {
			// TODO: Shift later elements forward or backwards.
			unimplemented!()
		}
	}
	
	fn get_zone(&self, index: usize) -> &[Vertex] {
		let zone = &self.zones[index].0;
		
		&self.local[zone.start..zone.start+zone.size]
	}
	
	// TODO: Handle errors.
	pub fn update<F, C>(&mut self, factory: &mut F, encoder: &mut Encoder<R, C>) where F: Factory<R> + FactoryExt<R>, C: CommandBuffer<R> {
		//println!("Begin update");
		if self.local.len() > self.remote.len() {
			// Full update
			let (pages, other) = (self.local.len() / ALLOC_STEP, self.local.len() % ALLOC_STEP);
			let pages = pages + if other != 0 {1} else {0};
			
			//println!("Full update {} -> {}", self.remote.len(), pages * ALLOC_STEP);
			
			self.remote = factory.create_buffer(pages * ALLOC_STEP, Role::Vertex, Usage::Dynamic, TRANSFER_DST).unwrap();
			encoder.update_buffer(&self.remote, &self.local[..self.tail], 0).unwrap();
		} else {
			// Partial update
			for &mut (ref zone, ref mut dirty) in self.zones.iter_mut().filter(|&&mut (_, dirty)| dirty) {
				// TODO: Performance: Roll adjacent updates into a single update.
				//println!("Update partial: {:?}", zone);
				
				encoder.update_buffer(&self.remote, &self.local[zone.start..zone.start+zone.size], zone.start).unwrap();
				*dirty = false
			}
		}
		//println!("End update");
	}
	
	pub fn remote(&self) -> &Buffer<R, Vertex> {
		&self.remote
	}
	
	pub fn slice(&self) -> Slice<R> {
		Slice {
			start: 0,
			end: self.tail as u32,
			base_vertex: 0,
			instances: None,
			buffer: IndexBuffer::Auto
		}
	}
}

impl<R> Extend<Vertex> for ManagedBuffer<R> where R: Resources {
	fn extend<I>(&mut self, iter: I) where I: IntoIterator<Item=Vertex> {
		if let Some(zone) = self.zones.last_mut() {
			let old_len = self.local.len();
			
			self.local.extend(iter);
			let len = self.local.len() - old_len;
		
			zone.0.size += len;
			zone.1 = true;
			self.tail += len;
		} else {
			panic!("Tried to extend to a previously created zone, but there are no zones.");
		}
	}
}

impl<'a, R> Extend<&'a Vertex> for ManagedBuffer<R> where R: Resources {
	fn extend<I>(&mut self, iter: I) where I: IntoIterator<Item=&'a Vertex> {
		if let Some(zone) = self.zones.last_mut() {
			let old_len = self.local.len();
			
			self.local.extend(iter);
			let len = self.local.len() - old_len;
		
			zone.0.size += len;
			zone.1 = true;
			self.tail += len;
		} else {
			panic!("Tried to extend to a previously created zone, but there are no zones.");
		}
	}
}